use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};
use std::fs::File;
use std::io::prelude::*;

use flowrs::RuntimeConnectable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct FileWriterConfig {
    file: String,
}

#[derive(Clone)]
pub struct FileReaderConfig {
    file: String,
}

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct BinaryFileWriterNode {
    #[input]
    pub data_input: Input<Vec<u8>>,

    #[input]
    pub config_input: Input<FileWriterConfig>,

    current_config: Option<FileWriterConfig>,
}

impl BinaryFileWriterNode {
    pub fn new() -> Self {
        Self {
            data_input: Input::new(),
            config_input: Input::new(),
            current_config: None,
        }
    }
}

impl Node for BinaryFileWriterNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(cfg) = self.config_input.next() {
            self.current_config = Some(cfg);
        }

        if let Ok(data) = self.data_input.next() {
            if let Some(cfg) = &self.current_config {
                let mut buffer =
                    File::create(&cfg.file).map_err(|e| UpdateError::Other(e.into()))?;
                buffer
                    .write(&data)
                    .map_err(|e| UpdateError::Other(e.into()))?;
            }
        }

        Ok(())
    }
}

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct BinaryFileReaderNode {
    #[output]
    pub data_output: Output<Vec<u8>>,

    #[input]
    pub config_input: Input<FileReaderConfig>,
}

impl BinaryFileReaderNode {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            data_output: Output::new(change_observer),
            config_input: Input::new(),
        }
    }
}

impl Node for BinaryFileReaderNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(cfg) = self.config_input.next() {
            let mut buffer = File::open(&cfg.file).map_err(|e| UpdateError::Other(e.into()))?;
            let mut data: Vec<u8> = Vec::new();
            buffer
                .read_to_end(&mut data)
                .map_err(|e| UpdateError::Other(e.into()))?;

            self.data_output
                .send(data)
                .map_err(|e| UpdateError::Other(e.into()))?;
        }

        Ok(())
    }
}

#[test]
fn test_file_read_and_write() {
    let file: String = "test.bin".into();

    let mut reader = BinaryFileReaderNode::new(None);
    let mut writer = BinaryFileWriterNode::new();

    let data_input: flowrs::connection::Edge<Vec<u8>> = flowrs::connection::Edge::new();

    flowrs::connection::connect(reader.data_output.clone(), data_input.clone());

    let data: Vec<u8> = "123".as_bytes().to_vec();

    let _ = writer
        .config_input
        .send(FileWriterConfig { file: file.clone() });
    let _ = writer.data_input.send(data.clone());
    let _ = writer.on_update();

    let _ = reader
        .config_input
        .send(FileReaderConfig { file: file.clone() });
    let _ = reader.on_update();

    if let Ok(res_data) = data_input.next() {
        tracing::debug!("{:?} {:?}", res_data, data);
        assert_eq!(res_data, data);
    } else {
        assert!(false);
    }

    let _ = std::fs::remove_file(file);

    //tracing::debug!("{:?}", odd_res_nums);
    //tracing::debug!("{:?}", even_res_nums);
}
