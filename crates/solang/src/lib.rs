// SPDX-License-Identifier: Apache-2.0
#![deny(clippy::all)]
#![deny(unsafe_code)]

pub mod abi;
pub mod codegen;
pub mod file_resolver;
pub mod standard_json;
pub mod languageserver;



// In Sema, we use result unit for returning early
// when code-misparses. The error will be added to the namespace diagnostics, no need to have anything but unit
// as error.
pub mod sema;
use tower_lsp::{
    jsonrpc::{Error, ErrorCode,},
   
        
    Client, LanguageServer, LspService, Server,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    path::PathBuf,
};
use tokio::sync::Mutex;
use crate::languageserver::{SolangServer, Files, GlobalCache};
use file_resolver::FileResolver;
use sema::diagnostics;
use solang_parser::pt;
use std::{ffi::OsStr, fmt};

/* 
use futures::stream::{TryStreamExt, IntoAsyncRead, };
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::stream::{JsStream,  };
use tokio::io::{AsyncRead, AsyncWriteExt, AsyncReadExt, };
use web_sys::{ReadableStream, WritableStream,  ReadableByteStreamController};*/



/// The target chain you want to compile Solidity for.
#[derive(Debug, Clone, Copy)]
pub enum Target {
    /// Solana, see <https://solana.com/>
    Solana,
    /// Parachains with the Substrate `contracts` pallet, see <https://substrate.io/>
    Polkadot {
        address_length: usize,
        value_length: usize,
    },
    /// Ethereum EVM, see <https://ethereum.org/en/developers/docs/evm/>
    EVM,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Target::Solana => write!(f, "Solana"),
            Target::Polkadot { .. } => write!(f, "Polkadot"),
            Target::EVM => write!(f, "EVM"),
        }
    }
}

impl PartialEq for Target {
    // Equality should check if it the same chain, not compare parameters. This
    // is needed for builtins for example
    fn eq(&self, other: &Self) -> bool {
        match self {
            Target::Solana => matches!(other, Target::Solana),
            Target::Polkadot { .. } => matches!(other, Target::Polkadot { .. }),
            Target::EVM => matches!(other, Target::EVM),
        }
    }
}

impl Target {
    /// Short-hand for checking for Polkadot target
    pub fn is_polkadot(&self) -> bool {
        matches!(self, Target::Polkadot { .. })
    }

    /// Create the target Polkadot with default parameters
    pub const fn default_polkadot() -> Self {
        Target::Polkadot {
            address_length: 32,
            value_length: 16,
        }
    }

    /// Creates a target from a string
    pub fn from(name: &str) -> Option<Self> {
        match name {
            "solana" => Some(Target::Solana),
            "polkadot" => Some(Target::default_polkadot()),
            "evm" => Some(Target::EVM),
            _ => None,
        }
    }

    /// File extension
    pub fn file_extension(&self) -> &'static str {
        match self {
            // Solana uses ELF dynamic shared object (BPF)
            Target::Solana => "so",
            // Everything else generates webassembly
            _ => "wasm",
        }
    }

    /// Size of a pointer in bits
    pub fn ptr_size(&self) -> u16 {
        if *self == Target::Solana {
            // Solana is BPF, which is 64 bit
            64
        } else {
            // All others are WebAssembly in 32 bit mode
            32
        }
    }

    /// This function returns the byte length for a selector, given the target
    pub fn selector_length(&self) -> u8 {
        match self {
            Target::Solana => 8,
            _ => 4,
        }
    }
}

/// Compile a solidity file to list of wasm files and their ABIs.
///
/// This function only produces a single contract and abi, which is compiled for the `target` specified. Any
/// compiler warnings, errors and informational messages are also provided.
///
/// The ctx is the inkwell llvm context.

/// Parse and resolve the Solidity source code provided in src, for the target chain as specified in target.
/// The result is a list of resolved contracts (if successful) and a list of compiler warnings, errors and
/// informational messages like `found contact N`.
///
/// Note that multiple contracts can be specified in on solidity source file.
pub fn parse_and_resolve(
    filename: &OsStr,
    resolver: &mut FileResolver,
    target: Target,
) -> sema::ast::Namespace {
    let mut ns = sema::ast::Namespace::new(target);

    match resolver.resolve_file(None, filename) {
        Err(message) => {
            ns.diagnostics.push(sema::ast::Diagnostic {
                ty: sema::ast::ErrorType::ParserError,
                level: sema::ast::Level::Error,
                message,
                loc: pt::Loc::CommandLine,
                notes: Vec::new(),
            });
        }
        Ok(file) => {
            sema::sema(&file, resolver, &mut ns);
        }
    }

    ns.diagnostics.sort_and_dedup();

    ns
}

/* 
#[wasm_bindgen]
pub fn trial() {
    // write to consol.log in browser
    console::log_1(&"Hello world!".into());
}*/


// example function to be called from javascript with a string argument
/*#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    //console::log_1(&format!("Hello, {}!", name).into());
     format!("Hello from Rust yazmeley {}!", name)
}*/


/* 
#[wasm_bindgen]
pub fn start_server(request: &str) -> String {
    languageserver::start_server(request)
}
*/


/* 
#[wasm_bindgen]
pub fn start_listener(stream: js_sys::AsyncIterator) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"server::serve".into());

    let sesa = stream.as_string().unwrap();
    println!("sema: {:?}", sesa);
    Ok(())
    
}*/






/* 
#[wasm_bindgen]
pub struct ServerConfig {
    into_server: js_sys::AsyncIterator,
    from_server: web_sys::WritableStream,
}

#[wasm_bindgen]
impl ServerConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(into_server: js_sys::AsyncIterator, from_server: web_sys::WritableStream) -> Self {
        Self {
            into_server,
            from_server,
        }
    }
}
*/

/* 

#[wasm_bindgen]
pub async fn start_server(config: ServerConfig) -> Result<(), JsValue>  {

    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"server::serve".into());

    let ServerConfig {
        into_server,
        from_server,
    } = config;

    //tree_sitter::TreeSitter::init().await?;
    //let language = demo_lsp_language::language::javascript().await.unwrap();

    let input = JsStream::from(into_server).into_stream();
    let tokio_input = tokio_util::compat::FuturesAsyncReadCompatExt::compat(input);
    //let input 

    let output = JsCast::unchecked_into::<wasm_streams::writable::sys::WritableStream>(from_server);
    let output = wasm_streams::WritableStream::from_raw(output);

    let output = output.try_into_async_write().map_err(|err| err.0)?;



    let mut importpaths = Vec::new();
    let mut importmaps = Vec::new();
let (service, socket) = LspService::new(|client| SolangServer {
    client,
    target: Target::Solana,
    importpaths,
    importmaps,
    files: Mutex::new(Files {
        caches: HashMap::new(),
        text_buffers: HashMap::new(),
    }),
    global_cache: Mutex::new(GlobalCache {
        definitions: HashMap::new(),
        types: HashMap::new(),
        implementations: HashMap::new(),
        declarations: HashMap::new(),
        properties: HashMap::new(),
    }),
});

let server = Server::new(input, output, socket).serve(service).await;

Ok(())
}
*/



/* 
#[wasm_bindgen]
pub async fn listen_and_echo() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"server::serve".into());

    

    

    Ok(())
}*/