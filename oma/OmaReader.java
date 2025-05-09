/*
 * OMA file format library for Rust
 * Copyright (c) 2023 Rust Port Author
 * Original Java code: de.kumakyoo.omalibjava
 *
 * This Rust library provides functionality for reading OMA format files,
 * which are used for storing OpenStreetMap data.
 */

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use flate2::read::ZlibDecoder;

/// The main reader for OMA format files.
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
package de.kumakyoo.omalibjava;

import java.io.*;
import java.util.*;
import java.util.zip.*;

public class OmaReader extends OmaTool {

    protected Filter filter;

    protected OmaInputStream save;

    protected BoundingBox globalBounds;

    protected Map<Byte, Map<String, Set<String>>> typeTable;

    protected boolean chunkFinished;
    protected int chunk;
    protected boolean blockFinished;
    protected int block;
    protected boolean sliceFinished;
    protected int slice;
    protected int elementCount;
    protected int element;

    protected String key;
    protected String value;

    public OmaReader(String filename) throws IOException {
        super(filename);
        setFilter(new Filter());
        key = value = null;

        openFile();
    }

    public void close() throws IOException {
        if (in != null) in.close();
    }

    public void reset() {
        chunkFinished = true;
        chunk = -1;
    }

    public Filter getFilter() {
        return filter;
    }

    public void setFilter(Filter filter) {
        this.filter = filter;
        reset();
    }

    public BoundingBox getBoundingBox() {
        return globalBounds;
    }

    public boolean containsBlocks(byte type, String key) {
        if (!typeTable.containsKey(type)) return false;
        return typeTable.get(type).containsKey(key);
    }

    public boolean containsSlices(byte type, String key, String value) {
        if (!typeTable.containsKey(type)) return false;
        if (!typeTable.get(type).containsKey(key)) return false;
        return typeTable.get(type).get(key).contains(value);
    }

    public Set<String> keySet(byte type) {
        if (!typeTable.containsKey(type)) return null;
        return typeTable.get(type).keySet();
    }

    public Set<String> valueSet(byte type, String key) {
        if (!typeTable.containsKey(type)) return null;
        if (!typeTable.get(type).containsKey(key)) return null;
        return typeTable.get(type).get(key);
    }

    public boolean isZipped() {
        return (features & 1) != 0;
    }

    public boolean containsID() {
        return (features & 2) != 0;
    }

    public boolean containsVersion() {
        return (features & 4) != 0;
    }

    public boolean containsTimestamp() {
        return (features & 8) != 0;
    }

    public boolean containsChangeset() {
        return (features & 16) != 0;
    }

    public boolean containsUser() {
        return (features & 32) != 0;
    }

    public boolean elementsOnce() {
        return (features & 64) != 0;
    }

    public Element next() throws IOException {
        while (true) {
            if (chunkFinished) if (!readNextChunk()) return null;

            if (
                !filter.needsChunk(
                    chunkTable[chunk].type,
                    chunkTable[chunk].bounds
                )
            ) {
                chunkFinished = true;
                continue;
            }

            if (blockFinished) if (!readNextBlock()) {
                chunkFinished = true;
                continue;
            }

            if (!filter.needsBlock(blockTable[block].key)) {
                blockFinished = true;
                continue;
            }

            if (sliceFinished) if (!readNextSlice()) {
                blockFinished = true;
                continue;
            }

            if (!filter.needsSlice(sliceTable[slice].value)) {
                in = save;
                sliceFinished = true;
                continue;
            }

            element++;
            if (element >= elementCount) {
                in = save;
                sliceFinished = true;
                continue;
            }

            Element e = readElement();
            if (filter.keep(e)) return e;
        }
    }

    public long count() throws IOException {
        long c = 0;

        for (chunk = 0; chunk < chunkTable.length; chunk++) {
            if (
                !filter.needsChunk(
                    chunkTable[chunk].type,
                    chunkTable[chunk].bounds
                )
            ) continue;

            readChunk();

            for (block = 0; block < blockTable.length; block++) {
                key = blockTable[block].key;
                if (!filter.needsBlock(key)) continue;

                readBlock();

                for (slice = 0; slice < sliceTable.length; slice++) {
                    value = sliceTable[slice].value;

                    if (!filter.needsSlice(value)) continue;

                    readSlice();

                    if (filter.countable()) c += elementCount;
                    else for (
                        element = 0;
                        element < elementCount;
                        element++
                    ) if (filter.keep(readElement())) c++;

                    in = save;
                }
            }
        }

        return c;
    }

    //////////////////////////////////////////////////////////////////

    protected void openFile() throws IOException {
        in = new OmaInputStream(filename);

        enforce(in.readByte() == 'O', "oma-file expected");
        enforce(in.readByte() == 'M', "oma-file expected");
        enforce(in.readByte() == 'A', "oma-file expected");
        enforce(in.readByte() == 0, "unknown version");

        features = in.readByte();

        globalBounds = new BoundingBox(
            in.readInt(),
            in.readInt(),
            in.readInt(),
            in.readInt()
        );

        long chunkTablePos = in.readLong();
        readTypeTable();
        in.setPosition(chunkTablePos);

        int count = in.readInt();
        chunkTable = new ChunkTableEntry[count];
        for (int i = 0; i < count; i++) chunkTable[i] = new ChunkTableEntry(
            in.readLong(),
            in.readByte(),
            new BoundingBox(in)
        );
    }

    protected void readTypeTable() throws IOException {
        OmaInputStream orig = in;
        if ((features & 1) != 0) in = new OmaInputStream(
            new BufferedInputStream(new InflaterInputStream(in))
        );

        typeTable = new HashMap<>();

        int count = in.readSmallInt();
        for (int i = 0; i < count; i++) {
            byte type = in.readByte();
            int count_keys = in.readSmallInt();

            Map<String, Set<String>> keyWithValues = new HashMap<>();

            for (int j = 0; j < count_keys; j++) {
                String key = in.readString();
                int count_values = in.readSmallInt();

                Set<String> values = new HashSet<>();

                for (int k = 0; k < count_values; k++) {
                    String value = in.readString();
                    values.add(value);
                }

                keyWithValues.put(key, values);
            }

            typeTable.put(type, keyWithValues);
        }

        in = orig;
    }

    protected boolean readNextChunk() throws IOException {
        chunkFinished = false;
        chunk++;
        if (chunk >= chunkTable.length) return false;
        readChunk();
        blockFinished = true;
        block = -1;
        return true;
    }

    protected void readChunk() throws IOException {
        in.setPosition(chunkTable[chunk].start);
        long blockTablePos = chunkTable[chunk].start + in.readInt();
        in.setPosition(blockTablePos);

        int count = in.readSmallInt();
        blockTable = new BlockTableEntry[count];
        for (int i = 0; i < count; i++) blockTable[i] = new BlockTableEntry(
            chunkTable[chunk].start + in.readInt(),
            in.readString()
        );
    }

    protected boolean readNextBlock() throws IOException {
        blockFinished = false;
        block++;
        if (block >= blockTable.length) return false;
        key = blockTable[block].key;

        readBlock();

        sliceFinished = true;
        slice = -1;

        return true;
    }

    protected void readBlock() throws IOException {
        in.setPosition(blockTable[block].start);
        long sliceTablePos = blockTable[block].start + in.readInt();
        in.setPosition(sliceTablePos);

        int count = in.readSmallInt();
        sliceTable = new SliceTableEntry[count];
        for (int i = 0; i < count; i++) sliceTable[i] = new SliceTableEntry(
            blockTable[block].start + in.readInt(),
            in.readString()
        );
    }

    protected boolean readNextSlice() throws IOException {
        sliceFinished = false;
        slice++;
        if (slice >= sliceTable.length) return false;
        value = sliceTable[slice].value;

        readSlice();
        element = -1;

        return true;
    }

    protected void readSlice() throws IOException {
        in.resetDelta();
        in.setPosition(sliceTable[slice].start);

        elementCount = in.readInt();
        save = in;
        if ((features & 1) != 0) in = new OmaInputStream(
            new BufferedInputStream(new InflaterInputStream(in))
        );
    }

    protected Element readElement() throws IOException {
        return readElement(chunk, key, value);
    }
}
