package de.kumakyoo.omalibjava;

import java.io.*;
import java.util.*;
import java.util.zip.*;

public class Extractor extends OmaTool
{
    protected int outCount;
    protected List<Filter> filter;
    protected List<String> name;
    protected boolean[] chunkUsed;
    protected boolean[] blockUsed;

    protected OmaOutputStream[] out;

    protected ChunkTableEntry[][] outChunkTable;
    protected BlockTableEntry[][] outBlockTable;
    protected SliceTableEntry[][] outSliceTable;

    public Extractor(String filename)
    {
        super(filename);
        filter = new ArrayList<>();
        name = new ArrayList<>();
    }

    public void addExtract(Filter f, String filename)
    {
        filter.add(f);
        name.add(filename);
    }

    public void run() throws IOException
    {
        outCount = filter.size();

        outChunkTable = new ChunkTableEntry[outCount][];
        outBlockTable = new BlockTableEntry[outCount][];
        outSliceTable = new SliceTableEntry[outCount][];

        openFiles();
        copyHeader();

        for (int i=0;i<outCount;i++)
            outChunkTable[i] = new ChunkTableEntry[chunkTable.length];

        for (int i=0;i<chunkTable.length;i++)
            extractChunk(i);

        for (int i=0;i<outCount;i++)
        {
            writeChunkTable(i);
            writeBoundingBox(i);
        }

        closeFiles();
    }

    protected void writeChunkTable(int i) throws IOException
    {
        long start = out[i].getPosition();
        int count = 0;
        for (ChunkTableEntry e: outChunkTable[i])
            if (e!=null)
                count++;
        out[i].writeInt(count);
        for (ChunkTableEntry e: outChunkTable[i])
            if (e!=null)
            {
                out[i].writeLong(e.start);
                out[i].writeByte(e.type);
                if (e.bounds==null)
                {
                    out[i].writeInt(Integer.MAX_VALUE);
                    out[i].writeInt(Integer.MAX_VALUE);
                    out[i].writeInt(Integer.MAX_VALUE);
                    out[i].writeInt(Integer.MAX_VALUE);
                }
                else
                {
                    out[i].writeInt(e.bounds.minlon);
                    out[i].writeInt(e.bounds.minlat);
                    out[i].writeInt(e.bounds.maxlon);
                    out[i].writeInt(e.bounds.maxlat);
                }
            }

        out[i].setPosition(21);
        out[i].writeLong(start);
    }

    protected void writeBoundingBox(int i) throws IOException
    {
        BoundingBox bounds = null;

        for (ChunkTableEntry e: outChunkTable[i])
            if (e!=null && e.bounds!=null)
            {
                if (bounds==null)
                    bounds = new BoundingBox(e.bounds);
                else
                {
                    bounds.minlon = Math.min(bounds.minlon,e.bounds.minlon);
                    bounds.minlat = Math.min(bounds.minlat,e.bounds.minlat);
                    bounds.maxlon = Math.max(bounds.maxlon,e.bounds.maxlon);
                    bounds.maxlat = Math.max(bounds.maxlat,e.bounds.maxlat);
                }
            }

        out[i].setPosition(5);
        if (bounds==null)
        {
            out[i].writeInt(Integer.MAX_VALUE);
            out[i].writeInt(Integer.MAX_VALUE);
            out[i].writeInt(Integer.MAX_VALUE);
            out[i].writeInt(Integer.MAX_VALUE);
        }
        else
        {
            out[i].writeInt(bounds.minlon);
            out[i].writeInt(bounds.minlat);
            out[i].writeInt(bounds.maxlon);
            out[i].writeInt(bounds.maxlat);
        }
    }

    protected void extractChunk(int chunk) throws IOException
    {
        chunkUsed = new boolean[outCount];
        boolean needed = false;
        for (int i=0;i<outCount;i++)
        {
            chunkUsed[i] = filter.get(i).needsChunk(chunkTable[chunk].type,chunkTable[chunk].bounds);
            if (chunkUsed[i])
            {
                outChunkTable[i][chunk] = new ChunkTableEntry(out[i].getPosition(),chunkTable[chunk].type,null);
                out[i].writeInt(0);
            }
            needed |= chunkUsed[i];
        }
        if (!needed) return;

        in.setPosition(chunkTable[chunk].start);
        long blockTablePos = chunkTable[chunk].start+in.readInt();
        in.setPosition(blockTablePos);

        int count = in.readSmallInt();
        blockTable = new BlockTableEntry[count];
        for (int i=0;i<count;i++)
            blockTable[i] = new BlockTableEntry(chunkTable[chunk].start+in.readInt(),in.readString());

        for (int i=0;i<outCount;i++)
            outBlockTable[i] = new BlockTableEntry[blockTable.length];

        for (int i=0;i<count;i++)
            extractBlock(chunk,i);

        for (int i=0;i<outCount;i++)
            if (chunkUsed[i])
                if (!writeBlockTable(i,chunk))
                    outChunkTable[i][chunk] = null;
    }

    protected boolean writeBlockTable(int i, int chunk) throws IOException
    {
        long tablepos = out[i].getPosition();
        out[i].setPosition(outChunkTable[i][chunk].start);

        int count = 0;
        for (BlockTableEntry e: outBlockTable[i])
            if (e!=null)
                count++;
        if (count==0) return false;

        out[i].writeInt((int)(tablepos-outChunkTable[i][chunk].start));
        out[i].setPosition(tablepos);

        out[i].writeSmallInt(count);
        for (BlockTableEntry e: outBlockTable[i])
            if (e!=null)
            {
                out[i].writeInt((int)(e.start-outChunkTable[i][chunk].start));
                out[i].writeString(e.key);
            }
        return true;
    }

    protected void extractBlock(int chunk, int block) throws IOException
    {
        blockUsed = new boolean[outCount];
        boolean needed = false;
        for (int i=0;i<outCount;i++)
        {
            blockUsed[i] = chunkUsed[i] && filter.get(i).needsBlock(blockTable[block].key);
            if (blockUsed[i])
            {
                outBlockTable[i][block] = new BlockTableEntry(out[i].getPosition(),blockTable[block].key);
                out[i].writeInt(0);
            }
            needed |= blockUsed[i];
        }
        if (!needed) return;

        in.setPosition(blockTable[block].start);
        long sliceTablePos = blockTable[block].start+in.readInt();
        in.setPosition(sliceTablePos);

        int count = in.readSmallInt();
        sliceTable = new SliceTableEntry[count];
        for (int i=0;i<count;i++)
            sliceTable[i] = new SliceTableEntry(blockTable[block].start+in.readInt(),in.readString());

        for (int i=0;i<outCount;i++)
            if (blockUsed[i])
                outSliceTable[i] = new SliceTableEntry[sliceTable.length];

        for (int i=0;i<count;i++)
            extractSlice(chunk,block,i);

        for (int i=0;i<outCount;i++)
            if (blockUsed[i])
                if (!writeSliceTable(i,chunk,block))
                    outBlockTable[i][block] = null;
    }

    protected boolean writeSliceTable(int i, int chunk, int block) throws IOException
    {
        long tablepos = out[i].getPosition();
        out[i].setPosition(outBlockTable[i][block].start);

        int count = 0;
        for (SliceTableEntry e: outSliceTable[i])
            if (e!=null)
                count++;
        if (count==0) return false;

        out[i].writeInt((int)(tablepos-outBlockTable[i][block].start));
        out[i].setPosition(tablepos);

        out[i].writeSmallInt(count);
        for (SliceTableEntry e: outSliceTable[i])
            if (e!=null)
            {
                out[i].writeInt((int)(e.start-outBlockTable[i][block].start));
                out[i].writeString(e.value);
            }
        return true;
    }

    protected void extractSlice(int chunk, int block, int slice) throws IOException
    {
        boolean[] sliceUsed = new boolean[outCount];
        boolean needed = false;
        for (int i=0;i<outCount;i++)
        {
            sliceUsed[i] = blockUsed[i] && filter.get(i).needsSlice(sliceTable[slice].value);
            if (sliceUsed[i])
                outSliceTable[i][slice] = new SliceTableEntry(out[i].getPosition(),sliceTable[slice].value);
            needed |= sliceUsed[i];
        }
        if (!needed) return;

        in.resetDelta();
        in.setPosition(sliceTable[slice].start);

        int elementCount = in.readInt();
        OmaInputStream save = in;
        if ((features&1)!=0)
            in = new OmaInputStream(new BufferedInputStream(new InflaterInputStream(in)));

        int[] count = new int[outCount];
        OmaOutputStream[] orig = new OmaOutputStream[outCount];
        DeflaterOutputStream[] dos = new DeflaterOutputStream[outCount];
        BufferedOutputStream[] bos = new BufferedOutputStream[outCount];

        for (int i=0;i<outCount;i++)
        {
            orig[i] = out[i];
            out[i].resetDelta();
        }

        for (int i=0;i<elementCount;i++)
        {
            Element e = readElement(chunk,blockTable[block].key,sliceTable[slice].value);
            for (int j=0;j<outCount;j++)
                if (sliceUsed[j])
                {
                    if (filter.get(j).keep(e))
                    {
                        if (count[j]==0)
                        {
                            out[j].writeInt(0);
                            if ((features&1)!=0)
                            {
                                dos[j] = new DeflaterOutputStream(out[j], new Deflater(Deflater.BEST_COMPRESSION));
                                bos[j] = new BufferedOutputStream(dos[j]);
                                out[j] = new OmaOutputStream(bos[j]);
                            }
                        }
                        count[j]++;
                        e.write(out[j],features|(chunkTable[chunk].type=='C'?2:0));
                        adjustBoundingBoxOfChunk(j,chunk,e);
                    }
                }
        }

        for (int i=0;i<outCount;i++)
        {
            if (count[i]>0)
            {
                if ((features&1)!=0)
                {
                    bos[i].flush();
                    dos[i].finish();
                    out[i] = orig[i];
                }

                long end = out[i].getPosition();
                out[i].setPosition(outSliceTable[i][slice].start);
                out[i].writeInt(count[i]);
                out[i].setPosition(end);
            }
            else if (blockUsed[i])
                outSliceTable[i][slice] = null;
        }

        in = save;
    }

    protected void adjustBoundingBoxOfChunk(int c, int chunk, Element e)
    {
        switch (chunkTable[chunk].type)
        {
        case 'N':
            Node n = (Node)e;
            adjustBoundingBoxOfChunk(c,chunk,n.lon,n.lat);
            break;
        case 'W':
            Way w = (Way)e;
            for (int i=0;i<w.lon.length;i++)
                adjustBoundingBoxOfChunk(c,chunk,w.lon[i],w.lat[i]);
            break;
        case 'A':
            Area a = (Area)e;
            for (int i=0;i<a.lon.length;i++)
                adjustBoundingBoxOfChunk(c,chunk,a.lon[i],a.lat[i]);
            // we can skip holes
            break;
        case 'C':
            Collection col = (Collection)e;
            // nothing to do yet...
            break;
        }
    }

    protected void adjustBoundingBoxOfChunk(int c, int chunk, int lon, int lat)
    {
        if (outChunkTable[c][chunk].bounds==null)
            outChunkTable[c][chunk].bounds = new BoundingBox(lon,lat,lon,lat);
        else
        {
            outChunkTable[c][chunk].bounds.minlon = Math.min(outChunkTable[c][chunk].bounds.minlon,lon);
            outChunkTable[c][chunk].bounds.minlat = Math.min(outChunkTable[c][chunk].bounds.minlat,lat);
            outChunkTable[c][chunk].bounds.maxlon = Math.max(outChunkTable[c][chunk].bounds.maxlon,lon);
            outChunkTable[c][chunk].bounds.maxlat = Math.max(outChunkTable[c][chunk].bounds.maxlat,lat);
        }
    }

    protected void openFiles() throws IOException
    {
        in = new OmaInputStream(filename);

        out = new OmaOutputStream[outCount];
        for (int i=0;i<outCount;i++)
            out[i] = new OmaOutputStream(name.get(i));
    }

    protected void closeFiles() throws IOException
    {
        in.close();
        for (int i=0;i<outCount;i++)
            out[i].close();
    }

    protected void copyHeader() throws IOException
    {
        enforce(in.readByte()=='O', "oma-file expected");
        enforce(in.readByte()=='M', "oma-file expected");
        enforce(in.readByte()=='A', "oma-file expected");
        enforce(in.readByte()==0, "unknown version");

        for (int i=0;i<outCount;i++)
        {
            out[i].writeByte('O');
            out[i].writeByte('M');
            out[i].writeByte('A');
            out[i].writeByte(0);
        }

        features = in.readByte();
        for (int i=0;i<outCount;i++)
        {
            out[i].writeByte(features);

            // writing bounds deferred
            out[i].writeInt(-1800000000);
            out[i].writeInt(1800000000);
            out[i].writeInt(-900000000);
            out[i].writeInt(900000000);

            // writing chunktablepos deferred
            out[i].writeLong(0);
        }

        // we do not need input bounding box
        in.readInt();
        in.readInt();
        in.readInt();
        in.readInt();

        long chunkTablePos = in.readLong();
        copyTypeTable();

        in.setPosition(chunkTablePos);

        int count = in.readInt();
        chunkTable = new ChunkTableEntry[count];
        for (int i=0;i<count;i++)
            chunkTable[i] = new ChunkTableEntry(in.readLong(),in.readByte(),new BoundingBox(in));
    }

    protected void copyTypeTable() throws IOException
    {
        OmaOutputStream[] origout = new OmaOutputStream[outCount];
        DeflaterOutputStream[] dos = new DeflaterOutputStream[outCount];
        BufferedOutputStream[] bos = new BufferedOutputStream[outCount];

        OmaInputStream orig = in;
        for (int i=0;i<outCount;i++)
            origout[i] = out[i];

        if ((features&1)!=0)
        {
            in = new OmaInputStream(new BufferedInputStream(new InflaterInputStream(in)));

            for (int i=0;i<outCount;i++)
            {
                dos[i] = new DeflaterOutputStream(out[i], new Deflater(Deflater.BEST_COMPRESSION));
                bos[i] = new BufferedOutputStream(dos[i]);
                out[i] = new OmaOutputStream(bos[i]);
            }
        }

        int count = in.readSmallInt();
        for (int l=0;l<outCount;l++)
            out[l].writeSmallInt(count);
        for (int i=0;i<count;i++)
        {
            byte type = in.readByte();
            int count_keys = in.readSmallInt();
            for (int l=0;l<outCount;l++)
            {
                out[l].writeByte(type);
                out[l].writeSmallInt(count_keys);
            }

            for (int j=0;j<count_keys;j++)
            {
                String key = in.readString();
                int count_values = in.readSmallInt();
                for (int l=0;l<outCount;l++)
                {
                    out[l].writeString(key);
                    out[l].writeSmallInt(count_values);
                }

                for (int k=0;k<count_values;k++)
                {
                    String value = in.readString();
                    for (int l=0;l<outCount;l++)
                        out[l].writeString(value);
                }
            }
        }

        if ((features&1)!=0)
            for (int i=0;i<outCount;i++)
            {
                bos[i].flush();
                dos[i].finish();
                out[i] = origout[i];
            }
        in = orig;
    }
}
