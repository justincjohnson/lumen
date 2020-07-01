use core::ffi::c_void;
use core::mem;
use core::slice;

use hashbrown::HashMap;

use once_cell::sync::OnceCell;

use liblumen_arena::DroplessArena;
use liblumen_core::symbols::FunctionSymbol;
#[cfg(all(unix, target_arch = "x86_64"))]
use liblumen_core::sys::dynamic_call;
use liblumen_core::sys::dynamic_call::DynamicCallee;

use crate::erts::term::prelude::Atom;
#[cfg(all(unix, target_arch = "x86_64"))]
use crate::erts::term::prelude::{Encoded, Term};
use crate::erts::ModuleFunctionArity;

/// Dynamically invokes the function mapped to the given symbol.
///
/// - The caller is responsible for making sure that the given symbol
/// belongs to a function compiled into the executable.
/// - The caller must ensure that the target function adheres to the ABI
/// requirements of the destination function:
///   - C calling convention
///   - Accepts only immediate-sized terms as arguments
///   - Returns an immediate-sized term as a result
///
/// This function returns `Err` if the called function returns the NONE value,
/// or if the given symbol doesn't exist.
///
/// This function will panic if the symbol table has not been initialized.
#[cfg(all(unix, target_arch = "x86_64"))]
pub unsafe fn apply(symbol: &ModuleFunctionArity, args: &[Term]) -> Result<Term, ()> {
    if let Some(f) = find_symbol(symbol) {
        let argv = args.as_ptr() as *const usize;
        let argc = args.len();
        let result = mem::transmute::<usize, Term>(dynamic_call::apply(f, argv, argc));
        if result.is_none() {
            Err(())
        } else {
            Ok(result)
        }
    } else {
        Err(())
    }
}

pub fn find_symbol(mfa: &ModuleFunctionArity) -> Option<DynamicCallee> {
    let symbols = SYMBOLS.get().unwrap_or_else(|| {
        panic!(
            "InitializeLumenDispatchTable not called before trying to get {:?}",
            mfa
        )
    });
    if let Some(f) = symbols.get_function(mfa) {
        Some(unsafe { mem::transmute::<*const c_void, DynamicCallee>(f) })
    } else {
        None
    }
}

pub fn dump_symbols() {
    let symbols = unsafe { SYMBOLS.get_unchecked() };
    symbols.dump();
}

/// The symbol table used by the runtime system
static SYMBOLS: OnceCell<SymbolTable> = OnceCell::new();

/// Performs one-time initialization of the atom table at program start, using the
/// array of constant atom values present in the compiled program.
///
/// It is expected that this will be called by code generated by the compiler, during the
/// earliest phase of startup, to ensure that nothing has tried to use the atom table yet.
#[no_mangle]
pub unsafe extern "C" fn InitializeLumenDispatchTable(
    table: *const FunctionSymbol,
    len: usize,
) -> bool {
    if table.is_null() {
        return false;
    }
    let raw_table = slice::from_raw_parts::<'static>(table, len);

    match SymbolTable::from_raw(raw_table) {
        Err(err) => {
            eprintln!("Error: {}", err);
            false
        }
        Ok(sym_table) => {
            if let Err(_) = SYMBOLS.set(sym_table) {
                eprintln!("tried to initialize symbol table more than once!");
                false
            } else {
                true
            }
        }
    }
}

struct SymbolTable {
    functions: HashMap<&'static ModuleFunctionArity, *const c_void>,
    idents: HashMap<*const c_void, &'static ModuleFunctionArity>,
    arena: DroplessArena,
}
impl SymbolTable {
    fn new(size: usize) -> Self {
        Self {
            functions: HashMap::with_capacity(size),
            idents: HashMap::with_capacity(size),
            arena: DroplessArena::default(),
        }
    }

    fn dump(&self) {
        eprintln!("START SymbolTable at {:p}", self);
        for mfa in self.functions.keys() {
            eprintln!("{:?}", mfa);
        }
        eprintln!("END SymbolTable");
    }

    /// Used to initialize the atom table from an array of null-terminated strings with static
    /// lifetime, such as generated by codegen in order to initialize the atom table with
    /// constant atom values in the compiled program. It is expected that this will be called
    /// via `InitializeLumenAtomTable`
    unsafe fn from_raw(raw_table: &'static [FunctionSymbol]) -> anyhow::Result<Self> {
        let mut table = Self::new(raw_table.len());

        for FunctionSymbol {
            module,
            function,
            arity,
            ptr,
        } in raw_table.iter()
        {
            // This is safe because the underlying data is static
            let module = Atom::from_id(*module);
            let function = Atom::from_id(*function);
            let callee = *ptr;
            let size = mem::size_of::<ModuleFunctionArity>();
            let ptr = table
                .arena
                .alloc_raw(size, mem::align_of::<ModuleFunctionArity>())
                as *mut ModuleFunctionArity;
            ptr.write(ModuleFunctionArity {
                module,
                function,
                arity: *arity,
            });
            let sym = mem::transmute::<&ModuleFunctionArity, &'static ModuleFunctionArity>(&*ptr);
            assert_eq!(None, table.idents.insert(callee, sym));
            assert_eq!(None, table.functions.insert(sym, callee));
        }

        Ok(table)
    }

    #[allow(unused)]
    fn get_ident(&self, function: *const c_void) -> Option<&'static ModuleFunctionArity> {
        self.idents.get(&function).copied()
    }

    fn get_function(&self, ident: &ModuleFunctionArity) -> Option<*const c_void> {
        self.functions.get(ident).copied()
    }
}

// These are safe to implement because the items in the symbol table are static
unsafe impl Sync for SymbolTable {}
unsafe impl Send for SymbolTable {}
