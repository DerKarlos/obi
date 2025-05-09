package de.kumakyoo.omalibjava;

import java.io.*;

abstract class OmaTool
{
    protected String filename;
    protected OmaInputStream in;

    protected ChunkTableEntry[] chunkTable;
    protected BlockTableEntry[] blockTable;
    protected SliceTableEntry[] sliceTable;

    protected int features;

    protected OmaTool(String filename)
    {
        this.filename = filename;
    }

    protected Element readElement(int chunk, String key, String value) throws IOException
    {
        Element e = null;

        switch (chunkTable[chunk].type)
        {
        case 'N':
            e = new Node(in,key,value);
            break;
        case 'W':
            e = new Way(in,key,value);
            break;
        case 'A':
            e = new Area(in,key,value);
            break;
        case 'C':
            e = new Collection(in,key,value);
            break;
        default:
            enforce(false, "unknown element type '"+((char)chunkTable[chunk].type)+"'");
        }

        e.readTags(in);
        e.readMembers(in);
        e.readMeta(in,features|(chunkTable[chunk].type=='C'?2:0));

        return e;
    }

    protected void enforce(boolean b, String msg) throws IOException
    {
        if (!b) throw new IOException(msg);
    }

    protected class ChunkTableEntry
    {
        public long start;
        public byte type;
        public BoundingBox bounds;

        public ChunkTableEntry(long start, byte type, BoundingBox bounds)
        {
            this.start = start;
            this.type = type;
            this.bounds = bounds;
        }
    }

    protected class BlockTableEntry
    {
        public long start;
        public String key;

        public BlockTableEntry(long start, String key)
        {
            this.start = start;
            this.key = key;
        }
    }

    protected class SliceTableEntry
    {
        public long start;
        public String value;

        public SliceTableEntry(long start, String value)
        {
            this.start = start;
            this.value = value;
        }
    }
}
