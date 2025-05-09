package de.kumakyoo.omalibjava;

import java.io.*;
import java.nio.channels.*;
import java.util.*;

public class OmaInputStream extends DataInputStream
{
    private FileInputStream fis;
    private FileChannel fc;

    private int lastx;
    private int lasty;

    public OmaInputStream(String filename) throws IOException
    {
        super(null);

        fis = new FileInputStream(filename);
        fc = fis.getChannel();
        in = new BufferedInputStream(fis);
        resetDelta();
    }

    public OmaInputStream(InputStream in) throws IOException
    {
        super(in);

        fis = null;
        fc = null;
        resetDelta();
    }

    //////////////////////////////////////////////////////////////////

    public int readSmallInt() throws IOException
    {
        int val = readUnsignedByte();
        if (val<255) return val;
        val = readUnsignedShort();
        if (val<65535) return val;
        return readInt();
    }

    public String readString() throws IOException
    {
        int len = readSmallInt();
        byte[] b = new byte[len];
        readFully(b,0,len);
        return new String(b,"UTF-8");
    }

    public void resetDelta()
    {
        lastx = lasty = 0;
    }

    public int readDeltaX() throws IOException
    {
        lastx = delta(lastx);
        return lastx;
    }

    public int readDeltaY() throws IOException
    {
        lasty = delta(lasty);
        return lasty;
    }

    private int delta(int last) throws IOException
    {
        int delta = readShort();
        return delta==-32768?readInt():(last+delta);
    }

    //////////////////////////////////////////////////////////////////

    // note: This returns only a rough estimate due to usage of buffers
    public long getPosition() throws IOException
    {
        return fc==null?0:fc.position();
    }

    public void setPosition(long pos) throws IOException
    {
        if (fc==null) return;
        fc.position(pos);
        in = new BufferedInputStream(fis);
    }
}
