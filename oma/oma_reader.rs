/*
 * OMA file format library for Rust
 * Copyright (c) 2023 Rust Port Author
 * Original Java code: de.kumakyoo.omalibjava
 *
 * This Rust library provides functionality for reading OMA format files,
 * which are used for storing OpenStreetMap data.
 */

//package de.kumakyoo.omalibjava;

//import java.io.*;
//import java.util.*;
//import java.util.zip.*;

use flate2::read::ZlibDecoder;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// The main reader for OMA format files.
//public class OmaReader extends OmaTool {impl OmaReader {
pub struct OmaReader {
    filter: Filter,
    save: Option<OmaInputStream>,
    global_bounds: BoundingBox,
    type_table: HashMap<u8, HashMap<String, HashSet<String>>>,

    chunk_finished: bool,
    chunk: usize,
    block_finished: bool,
    block: usize,
    slice_finished: bool,
    slice: usize,
    element_count: usize,
    element: usize,

    key: Option<String>,
    value: Option<String>,

    // Base fields from OmaTool
    file: Option<File>,
    filename: String,
    features: u8,
    chunk_table: Vec<ChunkTableEntry>,
    block_table: Vec<BlockTableEntry>,
    slice_table: Vec<SliceTableEntry>,
}

//extends OmaTool {impl OmaReader
impl OmaReader {
    /// Creates a new OMA reader with the given filename
    pub fn new(filename: String) -> io::Result<Self> {
        let mut reader = OmaReader {
            filter: Filter::new(),
            save: None,
            global_bounds: BoundingBox::default(),
            type_table: HashMap::new(),

            chunk_finished: true,
            chunk: 0,
            block_finished: true,
            block: 0,
            slice_finished: true,
            slice: 0,
            element_count: 0,
            element: 0,

            key: None,
            value: None,

            file: None,
            filename,
            features: 0,
            chunk_table: Vec::new(),
            block_table: Vec::new(),
            slice_table: Vec::new(),
        };

        reader.open_file()?;
        Ok(reader)
    }

    /// Closes the reader and its underlying file
    pub fn close(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.file {
            file.flush()?;
        }
        self.file = None;
        Ok(())
    }

    /// Resets the reader to the beginning
    pub fn reset(&mut self) {
        self.chunk_finished = true;
        self.chunk = 0;
    }

    /// Gets the current filter
    pub fn get_filter(&self) -> &Filter {
        &self.filter
    }

    /// Sets a new filter and resets the reader
    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
        self.reset();
    }

    /// Gets the bounding box of the OMA file
    pub fn get_bounding_box(&self) -> &BoundingBox {
        &self.global_bounds
    }

    /// Checks if the given type contains blocks with the given key
    pub fn contains_blocks(&self, type_id: u8, key: &str) -> bool {
        if !self.type_table.contains_key(&type_id) {
            return false;
        }
        self.type_table.get(&type_id).unwrap().contains_key(key)
    }

    /// Checks if the given type contains slices with the given key/value pair
    pub fn contains_slices(&self, type_id: u8, key: &str, value: &str) -> bool {
        if !self.type_table.contains_key(&type_id) {
            return false;
        }
        let key_map = self.type_table.get(&type_id).unwrap();
        if !key_map.contains_key(key) {
            return false;
        }
        key_map.get(key).unwrap().contains(value)
    }

    /// Gets all keys for the given type
    pub fn key_set(&self, type_id: u8) -> Option<Vec<String>> {
        if !self.type_table.contains_key(&type_id) {
            return None;
        }
        let keys: Vec<String> = self
            .type_table
            .get(&type_id)
            .unwrap()
            .keys()
            .cloned()
            .collect();
        Some(keys)
    }

    /// Gets all values for the given type and key
    pub fn value_set(&self, type_id: u8, key: &str) -> Option<Vec<String>> {
        if !self.type_table.contains_key(&type_id) {
            return None;
        }
        let key_map = self.type_table.get(&type_id).unwrap();
        if !key_map.contains_key(key) {
            return None;
        }
        let values: Vec<String> = key_map.get(key).unwrap().iter().cloned().collect();
        Some(values)
    }

    /// Checks if the file is compressed
    pub fn is_zipped(&self) -> bool {
        (self.features & 1) != 0
    }

    /// Checks if the file contains IDs
    pub fn contains_id(&self) -> bool {
        (self.features & 2) != 0
    }

    /// Checks if the file contains version information
    pub fn contains_version(&self) -> bool {
        (self.features & 4) != 0
    }

    /// Checks if the file contains timestamps
    pub fn contains_timestamp(&self) -> bool {
        (self.features & 8) != 0
    }

    /// Checks if the file contains changeset information
    pub fn contains_changeset(&self) -> bool {
        (self.features & 16) != 0
    }

    /// Checks if the file contains user information
    pub fn contains_user(&self) -> bool {
        (self.features & 32) != 0
    }

    /// Checks if elements appear only once
    pub fn elements_once(&self) -> bool {
        (self.features & 64) != 0
    }

    /// Gets the next element that passes the filter
    pub fn next(&mut self) -> io::Result<Option<Element>> {
        loop {
            if self.chunk_finished {
                if !self.read_next_chunk()? {
                    return Ok(None);
                }
            }

            if !self.filter.needs_chunk(
                self.chunk_table[self.chunk].type_id,
                &self.chunk_table[self.chunk].bounds,
            ) {
                self.chunk_finished = true;
                continue;
            }

            if self.block_finished {
                if !self.read_next_block()? {
                    self.chunk_finished = true;
                    continue;
                }
            }

            if !self.filter.needs_block(&self.block_table[self.block].key) {
                self.block_finished = true;
                continue;
            }

            if self.slice_finished {
                if !self.read_next_slice()? {
                    self.block_finished = true;
                    continue;
                }
            }

            if !self.filter.needs_slice(&self.slice_table[self.slice].value) {
                self.save = None;
                self.slice_finished = true;
                continue;
            }

            self.element += 1;
            if self.element >= self.element_count {
                self.save = None;
                self.slice_finished = true;
                continue;
            }

            let element = self.read_element()?;
            if self.filter.keep(&element) {
                return Ok(Some(element));
            }
        }
    }

    /// Counts elements that pass the filter
    pub fn count(&mut self) -> io::Result<u64> {
        let mut c = 0;

        for chunk_idx in 0..self.chunk_table.len() {
            self.chunk = chunk_idx;

            if !self.filter.needs_chunk(
                self.chunk_table[self.chunk].type_id,
                &self.chunk_table[self.chunk].bounds,
            ) {
                continue;
            }

            self.read_chunk()?;

            for block_idx in 0..self.block_table.len() {
                self.block = block_idx;
                self.key = Some(self.block_table[self.block].key.clone());

                if !self.filter.needs_block(self.key.as_ref().unwrap()) {
                    continue;
                }

                self.read_block()?;

                for slice_idx in 0..self.slice_table.len() {
                    self.slice = slice_idx;
                    self.value = Some(self.slice_table[self.slice].value.clone());

                    if !self.filter.needs_slice(self.value.as_ref().unwrap()) {
                        continue;
                    }

                    self.read_slice()?;

                    if self.filter.countable() {
                        c += self.element_count as u64;
                    } else {
                        for element_idx in 0..self.element_count {
                            self.element = element_idx;
                            if self.filter.keep(&self.read_element()?) {
                                c += 1;
                            }
                        }
                    }

                    self.save = None;
                }
            }
        }

        Ok(c)
    }

    // Private implementation details

    fn open_file(&mut self) -> io::Result<()> {
        let mut file = File::open(&self.filename)?;
        let mut oma_input = OmaInputStream::new(file);

        // Check OMA signature
        self.enforce(oma_input.read_byte()? == b'O', "oma-file expected")?;
        self.enforce(oma_input.read_byte()? == b'M', "oma-file expected")?;
        self.enforce(oma_input.read_byte()? == b'A', "oma-file expected")?;
        self.enforce(oma_input.read_byte()? == 0, "unknown version")?;

        self.features = oma_input.read_byte()?;

        // Read global bounds
        self.global_bounds = BoundingBox::new(
            oma_input.read_int()?,
            oma_input.read_int()?,
            oma_input.read_int()?,
            oma_input.read_int()?,
        );

        let chunk_table_pos = oma_input.read_long()?;
        self.read_type_table(&mut oma_input)?;
        oma_input.set_position(chunk_table_pos)?;

        let count = oma_input.read_int()? as usize;
        self.chunk_table = Vec::with_capacity(count);

        for _ in 0..count {
            self.chunk_table.push(ChunkTableEntry::new(
                oma_input.read_long()?,
                oma_input.read_byte()?,
                BoundingBox::from_input(&mut oma_input)?,
            ));
        }

        self.file = Some(file);
        Ok(())
    }

    fn read_type_table(&mut self, input: &mut OmaInputStream) -> io::Result<()> {
        let mut orig = input.clone();
        let mut zipped_input = if (self.features & 1) != 0 {
            let decoder = ZlibDecoder::new(input);
            OmaInputStream::new(decoder)
        } else {
            input.clone()
        };

        self.type_table = HashMap::new();

        let count = zipped_input.read_small_int()? as usize;
        for _ in 0..count {
            let type_id = zipped_input.read_byte()?;
            let count_keys = zipped_input.read_small_int()? as usize;

            let mut key_with_values: HashMap<String, HashSet<String>> = HashMap::new();

            for _ in 0..count_keys {
                let key = zipped_input.read_string()?;
                let count_values = zipped_input.read_small_int()? as usize;

                let mut values: HashSet<String> = HashSet::new();

                for _ in 0..count_values {
                    let value = zipped_input.read_string()?;
                    values.insert(value);
                }

                key_with_values.insert(key, values);
            }

            self.type_table.insert(type_id, key_with_values);
        }

        *input = orig;
        Ok(())
    }

    fn read_next_chunk(&mut self) -> io::Result<bool> {
        self.chunk_finished = false;
        self.chunk += 1;
        if self.chunk >= self.chunk_table.len() {
            return Ok(false);
        }
        self.read_chunk()?;
        self.block_finished = true;
        self.block = 0;
        Ok(true)
    }

    fn read_chunk(&mut self) -> io::Result<()> {
        let mut oma_input = OmaInputStream::new(self.file.as_ref().unwrap());
        oma_input.set_position(self.chunk_table[self.chunk].start)?;

        let block_table_pos = self.chunk_table[self.chunk].start + oma_input.read_int()? as u64;
        oma_input.set_position(block_table_pos)?;

        let count = oma_input.read_small_int()? as usize;
        self.block_table = Vec::with_capacity(count);

        for _ in 0..count {
            self.block_table.push(BlockTableEntry::new(
                self.chunk_table[self.chunk].start + oma_input.read_int()? as u64,
                oma_input.read_string()?,
            ));
        }

        Ok(())
    }

    fn read_next_block(&mut self) -> io::Result<bool> {
        self.block_finished = false;
        self.block += 1;
        if self.block >= self.block_table.len() {
            return Ok(false);
        }
        self.key = Some(self.block_table[self.block].key.clone());

        self.read_block()?;

        self.slice_finished = true;
        self.slice = 0;

        Ok(true)
    }

    fn read_block(&mut self) -> io::Result<()> {
        let mut oma_input = OmaInputStream::new(self.file.as_ref().unwrap());
        oma_input.set_position(self.block_table[self.block].start)?;

        let slice_table_pos = self.block_table[self.block].start + oma_input.read_int()? as u64;
        oma_input.set_position(slice_table_pos)?;

        let count = oma_input.read_small_int()? as usize;
        self.slice_table = Vec::with_capacity(count);

        for _ in 0..count {
            self.slice_table.push(SliceTableEntry::new(
                self.block_table[self.block].start + oma_input.read_int()? as u64,
                oma_input.read_string()?,
            ));
        }

        Ok(())
    }

    fn read_next_slice(&mut self) -> io::Result<bool> {
        self.slice_finished = false;
        self.slice += 1;
        if self.slice >= self.slice_table.len() {
            return Ok(false);
        }
        self.value = Some(self.slice_table[self.slice].value.clone());

        self.read_slice()?;
        self.element = 0;

        Ok(true)
    }

    fn read_slice(&mut self) -> io::Result<()> {
        let mut oma_input = OmaInputStream::new(self.file.as_ref().unwrap());
        oma_input.reset_delta();
        oma_input.set_position(self.slice_table[self.slice].start)?;

        self.element_count = oma_input.read_int()? as usize;
        self.save = Some(oma_input.clone());

        if (self.features & 1) != 0 {
            let decoder = ZlibDecoder::new(&mut oma_input);
            oma_input = OmaInputStream::new(decoder);
        }

        Ok(())
    }

    fn read_element(&mut self) -> io::Result<Element> {
        self.read_element_with_context(
            self.chunk,
            self.key.as_ref().unwrap(),
            self.value.as_ref().unwrap(),
        )
    }

    fn read_element_with_context(
        &self,
        chunk: usize,
        key: &str,
        value: &str,
    ) -> io::Result<Element> {
        // Implementation depends on Element type
        unimplemented!("Element reading is not implemented")
    }

    fn enforce(&self, condition: bool, message: &str) -> io::Result<()> {
        if condition {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, message))
        }
    }
}
