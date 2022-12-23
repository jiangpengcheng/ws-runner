use wasmedge_sdk::config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions};
use wasmedge_sdk::error::HostFuncError;
use wasmedge_sdk::{host_function, Caller, ImportObjectBuilder, Vm, WasmValue};

static mut DATA: &str = "";

#[host_function]
pub unsafe fn get_length(
    _caller: Caller,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, HostFuncError> {
    unsafe { Ok(vec![WasmValue::from_i32(DATA.len() as i32)]) }
}

#[host_function]
pub fn result_buffer(
    caller: Caller,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, HostFuncError> {
    let pointer = _args[0].to_i32();
    println!("pointer: {}", pointer);
    let length = _args[1].to_i32();
    println!("length: {}", length);
    let result = caller
        .memory(0)
        .unwrap()
        .read(pointer as u32, length as u32)
        .unwrap();
    println!("result: {}", String::from_utf8(result).unwrap());
    Ok(vec![])
}

#[host_function]
pub unsafe fn write_to_buffer(
    caller: Caller,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, HostFuncError> {
    let pointer = _args[0].to_i32();
    println!("pointer: {}", pointer);
    unsafe {
        caller
            .memory(0)
            .unwrap()
            .write(DATA.as_bytes(), pointer as u32)
            .unwrap();
    }
    Ok(vec![])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create a config with `wasi` option enabled
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    // create a vm
    let import = ImportObjectBuilder::new()
        .with_func::<(), i32>("get_length", get_length)?
        .with_func::<(i32, i32), ()>("result_buffer", result_buffer)?
        .with_func::<i32, ()>("write_to_buffer", write_to_buffer)?
        .build("env")?;
    let vm = Vm::new(Some(config))?.register_import_module(import)?;

    unsafe { DATA = "hello world" }
    // load wasm module
    let wasm_file = std::path::PathBuf::from(".").join("demo.wasm");
    let res = vm.run_func_from_file(wasm_file, "run", vec![]);
    match res {
        Ok(v) => {
            println!("result: {:?}", v);
        }
        Err(e) => println!("error: {:?}", e),
    }

    Ok(())
}
