use crate::sketch::embedded::{delay, digital};
use anyhow::Context;
use gpio_cdev::{Chip, EventRequestFlags, LineRequestFlags};
use linux_embedded_hal::CdevPin;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{Read, Write};
use md5::compute;
use std::time::Duration;
use reqwest;
use serde_json::Value;
use async_std::task;
use wasmtime::{
    component::{bindgen, Component, Linker, ResourceTable},
    Config, Engine, Result, Store,
};

// Generate bindings of the guest and host components.
bindgen!({
    world: "blink",
    path: "../wit",
    with: {
        "sketch:embedded/delay/delay": Delay,
        "sketch:embedded/digital/input-pin": InputPin,
        "sketch:embedded/digital/output-pin": OutputPin,
    },
});

pub struct Delay;
pub struct InputPin(CdevPin);
pub struct OutputPin(CdevPin);

struct HostComponent {
    table: ResourceTable,
}

impl digital::Host for HostComponent {}
impl delay::Host for HostComponent {}

impl digital::HostInputPin for HostComponent {
    fn is_low(
        &mut self,
        self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<Result<bool, digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;
        match embedded_hal::digital::InputPin::is_low(&mut self_.0) {
            Ok(value) => Ok(Ok(value)),
            Err(_) => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn is_high(
        &mut self,
        self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<Result<bool, digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;
        match embedded_hal::digital::InputPin::is_high(&mut self_.0) {
            Ok(value) => Ok(Ok(value)),
            Err(_) => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn wait_for_high(
        &mut self,
        _self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        todo!("InputPin::wait_for_high")
    }

    fn wait_for_low(
        &mut self,
        _self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        todo!("InputPin::wait_for_low")
    }

    fn wait_for_rising_edge(
        &mut self,
        self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;
        let mut events = self_.0.line().events(
            LineRequestFlags::INPUT,
            EventRequestFlags::RISING_EDGE,
            "hello-embedded",
        )?;
        match events.next() {
            Some(Ok(_)) => Ok(Ok(())),
            _ => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn wait_for_falling_edge(
        &mut self,
        self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;
        let mut events = self_.0.line().events(
            LineRequestFlags::INPUT,
            EventRequestFlags::FALLING_EDGE,
            "hello-embedded",
        )?;
        match events.next() {
            Some(Ok(_)) => Ok(Ok(())),
            _ => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn wait_for_any_edge(
        &mut self,
        self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;
        let mut events = self_.0.line().events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES,
            "hello-embedded",
        )?;
        match events.next() {
            Some(Ok(_)) => Ok(Ok(())),
            _ => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn drop(
        &mut self,
        self_: wasmtime::component::Resource<digital::InputPin>,
    ) -> wasmtime::Result<()> {
        self.table.delete(self_)?;
        Ok(())
    }
}

impl digital::HostOutputPin for HostComponent {
    fn set_low(
        &mut self,
        self_: wasmtime::component::Resource<digital::OutputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;
        match embedded_hal::digital::OutputPin::set_low(&mut self_.0) {
            Ok(()) => Ok(Ok(())),
            Err(_) => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn set_high(
        &mut self,
        self_: wasmtime::component::Resource<digital::OutputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;
        match embedded_hal::digital::OutputPin::set_high(&mut self_.0) {
            Ok(()) => Ok(Ok(())),
            Err(_) => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn set_state(
        &mut self,
        self_: wasmtime::component::Resource<digital::OutputPin>,
        state: digital::PinState,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        let self_ = self.table.get_mut(&self_)?;

        let state = match state {
            digital::PinState::Low => embedded_hal::digital::PinState::Low,
            digital::PinState::High => embedded_hal::digital::PinState::High,
        };

        match embedded_hal::digital::OutputPin::set_state(&mut self_.0, state) {
            Ok(()) => Ok(Ok(())),
            Err(_) => Ok(Err(digital::ErrorCode::Other)),
        }
    }

    fn drop(
        &mut self,
        self_: wasmtime::component::Resource<digital::OutputPin>,
    ) -> wasmtime::Result<()> {
        self.table.delete(self_)?;
        Ok(())
    }
}

impl digital::HostStatefulOutputPin for HostComponent {
    fn is_set_high(
        &mut self,
        _self_: wasmtime::component::Resource<digital::StatefulOutputPin>,
    ) -> wasmtime::Result<Result<bool, digital::ErrorCode>> {
        todo!("StatefulOutputLin::is_set_high")
    }

    fn is_set_low(
        &mut self,
        _self_: wasmtime::component::Resource<digital::StatefulOutputPin>,
    ) -> wasmtime::Result<Result<bool, digital::ErrorCode>> {
        todo!("StatefulOutputLin::is_set_low")
    }

    fn toggle(
        &mut self,
        _self_: wasmtime::component::Resource<digital::StatefulOutputPin>,
    ) -> wasmtime::Result<Result<(), digital::ErrorCode>> {
        todo!("StatefulOutputLin::toggle")
    }

    fn drop(
        &mut self,
        self_: wasmtime::component::Resource<digital::StatefulOutputPin>,
    ) -> wasmtime::Result<()> {
        self.table.delete(self_)?;
        Ok(())
    }
}

impl delay::HostDelay for HostComponent {
    fn delay_ns(
        &mut self,
        self_: wasmtime::component::Resource<delay::Delay>,
        ns: u32,
    ) -> wasmtime::Result<()> {
        let _self_ = self.table.get_mut(&self_)?;
        std::thread::sleep(std::time::Duration::from_nanos(ns.into()));
        Ok(())
    }

    fn drop(&mut self, self_: wasmtime::component::Resource<delay::Delay>) -> wasmtime::Result<()> {
        self.table.delete(self_)?;
        Ok(())
    }
}

struct MyState {
    host: HostComponent,
}

fn main() -> Result<()> {
    task::spawn(async move {
        loop {
            task::sleep(Duration::from_secs(20)).await;
            check_update_available().await;
        }
    });
    // Create the engine and the linker.
    let engine = Engine::new(Config::new().wasm_component_model(true).wasm_multi_memory(true))?;
    let mut linker = Linker::<MyState>::new(&engine);

    Blink::add_to_linker(&mut linker, |state: &mut MyState| &mut state.host)?;
    loop {
    // Read the guest component file.
    let plugins = get_plugins_from_path("../active")?;
    if plugins.is_empty() {
        println!("No file present");
        return Ok(())
    }

    // Open the GPIO device.
    let mut chip = Chip::new("/dev/gpiochip0")
        .context("opening gpio device /dev/gpiochip0")?;

    // Create the state that will be stored in the store, and link it in.
    for plugin in plugins.iter() {
        let component_bytes = fs::read(plugin).context("failed to read input file")?;
        let component = Component::from_binary(&engine, &component_bytes)?;

        let mut my_state = MyState {
            host: HostComponent {
                table: ResourceTable::new(),
            },
        };

        // Request pin 17 as output.
        let output = CdevPin::new(chip.get_line(17)?.request(
            LineRequestFlags::OUTPUT,
            0,
            "write-output",
        )?)?;

        // Create the resources we'll pass into the `run` function.
        let led = my_state.host.table.push(OutputPin(output))?;
        let delay = my_state.host.table.push(Delay)?;

        // Create the store and instantiate the component.
        let mut store = Store::new(&engine, my_state);
        let (blink, _instance) = Blink::instantiate(&mut store, &component, &linker)?;

        // Run!
        blink
            .sketch_embedded_run()
            .call_run(&mut store, led, delay)?;
    }
    }
}

fn get_plugins_from_path(path: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut plugins = std::fs::read_dir(path)?
        .filter_map(|res| res.ok())
        .map(|dir_entry| dir_entry.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == "wasm") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    plugins.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(plugins)
}

async fn check_update_available() {
    let available = fetch_parse_input();
    if available {
        if let Err(err) = update_plugin() {
            eprintln!("Error updating file: {}", err);
        }
    } else {
        println!("File Error");
    }
}

fn update_plugin() -> Result<(), std::io::Error> {
    let file_name = find_latest_version("../staging");
    let staging_path = Path::new("../staging");
    let active_path = Path::new("../active");
    let file_path = staging_path.join(file_name.clone());
        if file_path.exists() {
            fs::copy(file_path, active_path.join("plugin.wasm"))?;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
        }
}

fn cleanup() {
    let latest_file = find_latest_version("../staging");
    match fs::remove_file(format!("{}/{}","../staging",latest_file)) {
        Ok(_) => println!("File at fault deleted successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn fetch_parse_input() -> bool {
    let ip_address = "localhost";                   // change ip address
    let file_name = find_latest_version("../staging");
    let version = trim_version(&file_name);

    let url = format!("http://{}:8080/checkupdate?current_version={}", ip_address, version);
    let response = match reqwest::blocking::get(url){
        Ok(response) => response,
        Err(err) => {
            println!("Error fetching CHECK-UPDATE URL: {}", err);
            return false
        }
    };
    let json_data = match response.text() {
        Ok(text) => text,
        Err(err) => {
            println!("Error parsing response: {}", err);
            return false
        }
    };
    let json_data = json_data.trim();
    let data: Value = match serde_json::from_str(&json_data) {
        Ok(data) => data,
        Err(err) => {
            println!("Error parsing JSON: {}", err);
            return false
        }
    };

    let latest_version = data["latest_version"].as_str().unwrap();
    let download_url = data["download_url"].as_str().unwrap();
    if !download_url.is_empty() {
        match download_file(ip_address, latest_version) {
            Ok(_) => (),
            Err(err) => println!("Error downloading file: {}", err),
        }
    }
    let check_sum = data["checksum"].as_str().unwrap();
    let latest_file = capture_filename_from_header(ip_address, latest_version);
    if is_valid_input(check_sum, latest_file.as_str()) {
        if is_wasm_file(latest_file.as_str()) {
            return true
        } else {
            cleanup();
            return false
        }
    } else { cleanup(); return false }
}

fn download_file(ip_address: &str, version: &str) -> Result<(), reqwest::Error> {
    // Construct the URL using the provided IP address and version
    let file_name = capture_filename_from_header(ip_address, version);
    if file_name.is_empty() { return Ok(()) }
    let url = format!("http://{}:8080/download?version={}", ip_address, version);
    let response = match reqwest::blocking::get(url) {
        Ok(response) => response,
        Err(err) => {
            return Err(err)
        }
    };
    if response.status().is_success() {
        let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{}/{}", "../staging", &file_name))
        .expect("Failed to open file");

        let body = response.bytes()?;
        let _ = file.write_all(&body);

        println!("File downloaded successfully: {}", file_name);
    } else {
        println!("Failed to download file: {}", response.status());
    }
    Ok(())
}

fn capture_filename_from_header(ip_address: &str, version: &str) -> String {
    let url = format!("http://{}:8080/download?version={}", ip_address, version);
    let response = match reqwest::blocking::get(url) {
        Ok(response) => response,
        Err(err) => {
            println!("Error fetching URL: {}", err);
            return "".to_string()
        }
    };
    let filename = response.headers().get("Content-Disposition").unwrap().to_str().unwrap();
    let filename = filename.split("=").last().unwrap().trim_matches('"');
    filename.to_string()
}

fn trim_version(file_name: &str) -> String {
    let parts: Vec<_> = file_name.split("_").collect();
    let version = parts.last().unwrap();
    let version = version.trim_start_matches("plugin_");
    version.to_string()
}

fn find_latest_version(dir_path: &str) -> String {
    let mut latest_version = (0, 0, 0);
    let mut latest_file_name = String::new();

    for entry in fs::read_dir(dir_path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let file_name: String = entry.file_name().to_string_lossy().into_owned();
        if file_name.starts_with("plugin_") && file_name.ends_with(".wasm") {
            let version = file_name.trim_start_matches("plugin_").trim_end_matches(".wasm");
            let version_parts: Vec<&str> = version.split('.').collect();
            if version_parts.len() == 3 {
                let major_version = version_parts[0].parse::<u32>().expect("Failed to parse major version");
                let minor_version = version_parts[1].parse::<u32>().expect("Failed to parse minor version");
                let patch_version = version_parts[2].parse::<u32>().expect("Failed to parse patch version");
                let current_version = (major_version, minor_version, patch_version);
                if latest_version < current_version {
                    latest_version = current_version;
                    latest_file_name = file_name;
                }
            }
        }
    }
    if latest_file_name.is_empty() {
        "No files found with the pattern 'plugin_X.Y.Z'".to_string()
    } else {
        latest_file_name
    }
}

fn is_valid_input(check_sum: &str, file_name: &str) -> bool {
    match File::open(format!("../staging/{}", file_name)){
        Ok(mut file) => {
                let mut buffer = Vec::new();
                let _ = file.read_to_end(&mut buffer);
                let md5_sum = compute(&buffer);
                let md5_sum = format!("{:x}", md5_sum);
                md5_sum == check_sum
            },
        Err(_) => false,
    }
}

fn is_wasm_file(file_name: &str) -> bool {
    let mut file = match File::open(format!("../staging/{}", file_name)) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut magic_number = [0; 4];
    match file.read_exact(&mut magic_number) {
        Ok(_) => (),
        Err(_) => return false,
    };
    magic_number == [0x00, 0x61, 0x73, 0x6d]
}
