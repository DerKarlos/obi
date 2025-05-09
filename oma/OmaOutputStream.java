package de.kumakyoo.omalibjava;

import java.io.*;
import java.nio.channels.*;
import java.util.*;

public class OmaOutputStream extends DataOutputStream
{
    private FileOutputStream fos;
    private FileChannel fc;

    private int lastx;
    private int lasty;

    public OmaOutputStream(String filename) throws IOException
    {
        super(null);

        fos = new FileOutputStream(filename);
        fc = fos.getChannel();
        out = new BufferedOutputStream(fos);

        resetDelta();
    }

    public OmaOutputStream(OutputStream s)
    {
        super(s);

        fos = null;
        fc = null;

        resetDelta();
    }

    //////////////////////////////////////////////////////////////////

    public void writeSmallInt(int value) throws IOException
    {
        if (value<255)
            writeByte(value);
        else
        {
            writeByte(255);
            if (value<65535)
                writeShort(value);
            else
            {
                writeShort(65535);
                writeInt(value);
            }
        }
    }

    public void writeString(String s) throws IOException
    {
        byte[] bytes = s.getBytes("UTF-8");
        writeSmallInt(bytes.length);
        write(bytes,0,bytes.length);
    }

    public void resetDelta()
    {
        lastx = lasty = 0;
    }

    public void writeDeltaX(int val) throws IOException
    {
        lastx = delta(lastx,val);
    }

    public void writeDeltaY(int val) throws IOException
    {
        lasty = delta(lasty,val);
    }

    private int delta(int last, int val) throws IOException
    {
        int delta = val-last;
        if (delta>=-32767 && delta<=32767)
            writeShort(delta);
        else
        {
            writeShort(-32768);
            writeInt(val);
        }

        return val;
    }

    //////////////////////////////////////////////////////////////////

    public long getPosition() throws IOException
    {
        if (fc==null) return 0;
        out.flush();
        return fc.position();
    }

    public void setPosition(long pos) throws IOException
    {
        if (fc==null) return;
        out.flush();
        fc.position(pos);
    }
}
