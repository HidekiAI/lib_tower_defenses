use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::sync::Mutex;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    string::String,
};

// Note: No need to drop/deconstruct/destroy once it's created
static RESOURCE_SINGLETON: Lazy<Mutex<ResourceFactory>> =
    Lazy::new(|| Mutex::new(ResourceFactory::new()));
struct ResourceFactory {
    resources: Vec<Resource>,
}

impl ResourceFactory {
    fn new() -> ResourceFactory {
        // Initialize your data here
        ResourceFactory {
            resources: Vec::new(),
        }
    }
}
pub type TResourceID = u16; // TODO: Move this to resource_system when available

//#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Resource {
    id: TResourceID,
    paths: String,
    buffer: Vec<u8>, // not for writing, mainly for reading?
}

impl Resource {
    // Note: Assume resource file will always exist (preprocessed/created)
    // in order to create new file (i.e. to save data), see create() impl
    // (write_data requires &Self, while/but create() does not)
    pub fn new(
        file_paths: String,
        allow_empty_file: bool,
    ) -> Result<TResourceID, Box<dyn std::error::Error>> {
        if file_paths.len() == 0 {
            return Err("file_paths passed is empty string (unspecified)".into());
        }
        let metadata_result = fs::metadata(file_paths.clone());
        let file_length = match metadata_result {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        };
        if (file_length == 0) && (allow_empty_file == false) {
            return Err(format!("file_paths '{}' passed is 0 bytes in length", file_paths).into());
        }

        let file = File::open(file_paths.to_owned()).unwrap();
        let mut reader = BufReader::new(file);
        let mut vec_buffer = Vec::new();
        match reader.read_to_end(&mut vec_buffer) {
            Ok(buff_size) => {
                let mut singleton = RESOURCE_SINGLETON.lock().unwrap();
                let max_id = match singleton.resources.iter().max_by_key(|e| e.id) { //.unwrap().id;
                    Some(o) => o.id,
                    _ => 0 as TResourceID,  // edge case when resources list is empty
                };
                let new_id = max_id + 1;
                if (vec_buffer.len() == 0 || buff_size == 0) && (allow_empty_file == false)
                {
                    return Err("File exists (was able to open), but buffer length is 0".into());
                }
                let ret_val = Resource {
                    id: new_id,
                    paths: file_paths,
                    buffer: vec_buffer.to_owned(),
                };
                singleton.resources.push(ret_val);
                Ok::<_, Box<dyn std::error::Error>>(new_id)
            }
            Err(e) =>
                // use result.downcast_ref::<io::Error>() such as with match{io::ErrorKind::NotFOund}.  i.e.
                //Err(io_error) => {
                //    // if file exists, throw a panic!(), else assume bin data does not exist, and create a brand new map
                //    println!("Error: {}", io_error.to_string());

                //    match io_error.downcast_ref::<io::Error>() {
                //        Some(io_casted_error) => match io_casted_error.kind() {
                //            io::ErrorKind::NotFound => do_something(),
                //            _ => Err(io_error.to_string()),
                //        },
                //        _ => Err(io_error.to_string()),
                //    }
                //}
                Err::<_, Box<dyn std::error::Error>>(Box::new(e)),
        }
    }

    pub fn create(
        file_paths: String,
        overwrite_if_exists: bool,
        //) -> Result<TResourceID, Box<dyn std::error::Error>> {
    ) -> Result<TResourceID, Box<dyn std::error::Error>> {
        return match OpenOptions::new()
            .write(true)
            .create_new(overwrite_if_exists == false)
            .open(file_paths.to_owned())
        {
            Ok(_file) => Resource::new(file_paths, true),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    if overwrite_if_exists {
                        match File::create(file_paths.clone()) {
                            Ok(_file2) => Resource::new(file_paths, true),
                            Err(e2) => Err(e2.into()),
                        }
                    } else {
                        Resource::new(file_paths, true)
                    }
                }
                std::io::ErrorKind::NotFound => match File::create(file_paths.clone()) {
                    Ok(_file2) => Resource::new(file_paths, true),
                    Err(e2) => Err(e2.into()),
                },
                _ => Err(e.into()),
            },
        };
    }
    pub fn get(res_id: TResourceID) -> Result<Resource, String> {
        let singleton = RESOURCE_SINGLETON.lock().unwrap();
        let found_index = singleton.resources.binary_search_by(|f| f.id.cmp(&res_id));
        match found_index {
            Ok(index) => Ok(singleton.resources[index].clone()), // clone for return
            Err(e) => Err(format!(
                "cannot locate index to resource_id={} - {}",
                res_id, e
            )),
        }
    }

    pub fn write_data<TF>(
        self: &mut Self,
        func_serialize_for_save: TF,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        TF: Fn() -> Result<Vec<u8>, String>,
    {
        match func_serialize_for_save() {
            Ok(serialized_buffer) => {
                //println!("Serialized {} bytes, begin writing...", result_of_T.len());
                let file = File::create(self.paths.to_owned())?;
                let mut writer = BufWriter::new(file);
                match writer.write_all(&serialized_buffer) {
                    Ok(()) => {
                        self.buffer = serialized_buffer.clone(); // update last read buffer with newly (and successfully) written buffer
                        let ret_result: Result<Vec<u8>, Box<dyn std::error::Error>> =
                            Ok(serialized_buffer.to_owned());
                        ret_result
                    }
                    Err(e) => Err::<Vec<u8>, Box<dyn std::error::Error>>(Box::new(e)),
                }
            }
            Err(e) => {
                println!("{}", e.to_string());
                let ret = Err("Call to resource::serialize_for_save failed".into());
                ret
            }
        }
    }

    pub fn read_data<T, TFn>(self: &Self, func_deserialize_for_load: TFn) -> Result<T, String>
    where
        TFn: Fn(&Vec<u8>) -> Result<T, String>,
    {
        //let file = File::open(fname)?;
        //let reader = BufReader::new(file);
        //let mut deserializer = Deserializer::new(reader);
        //let result: T = serde::Deserialize::deserialize(&mut deserializer)?;
        //return Ok(result);
        //let file = File::open(self.paths)?;
        //let mut reader = BufReader::new(file);
        //// read the whole file
        //let mut vec_buffer = Vec::new();
        //match reader.read_to_end(&mut vec_buffer) {
        //    Ok(_buff_size) => {
        //        let bin_buffer = vec_buffer.to_owned();
        //        let of_t_from_serialized_data = func_deserialize_for_load(&bin_buffer).unwrap();
        //        Ok::<T, Box<dyn std::error::Error>>(of_t_from_serialized_data)
        //    }
        //    Err(e) => Err::<T, Box<dyn std::error::Error>>(Box::new(e)),
        //}
        if self.buffer.len() == 0 {
            return Err("Buffer length for the resource is 0 bytes".to_owned());
        }
        match func_deserialize_for_load(&self.buffer) {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        }
    }

    //fn test_pass_fun() {
    //    fn apply_function<TFn>(a: i32, b: i32, func: TFn) -> i32
    //    where
    //        TFn: Fn(i32, i32) -> i32,
    //    {
    //        func(a, b)
    //    }
    //    let sum = |a, b| a + b;
    //    let product = |a, b| a * b;
    //    let diff = |a, b| a - b;
    //    println!("3 + 6 = {}", apply_function(3, 6, sum));
    //    println!("-4 * 9 = {}", apply_function(-4, 9, product));
    //    println!("7 - (-3) = {}", apply_function(7, -3, diff));
    //}
}
