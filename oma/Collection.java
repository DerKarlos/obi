package de.kumakyoo.omalibjava;

import java.io.*;

public class Collection extends Element
{
    public SliceDefinition[] defs;

    public Collection(OmaInputStream in, String key, String value) throws IOException
    {
        super(key,value);

        int count = in.readSmallInt();
        defs = new SliceDefinition[count];

        for (int i=0;i<count;i++)
            defs[i] = new SliceDefinition(in.readByte(),new BoundingBox(in.readInt(),in.readInt(),in.readInt(),in.readInt()),in.readString(),in.readString());
    }

    public void writeGeo(OmaOutputStream out) throws IOException
    {
        out.writeSmallInt(defs.length);
        for (SliceDefinition d:defs)
        {
            out.writeByte(d.type);
            out.writeInt(d.bounds.minlon);
            out.writeInt(d.bounds.minlat);
            out.writeInt(d.bounds.maxlon);
            out.writeInt(d.bounds.maxlat);
            out.writeString(d.key);
            out.writeString(d.value);
        }
    }

    public boolean isInside(Container c)
    {
        if (defs.length==0) return true;

        for (SliceDefinition d:defs)
            if (c.contains(d.bounds))
                return true;
        return false;
    }

    public String toString()
    {
        StringBuffer b = new StringBuffer();
        b.append("collection\n");
        for (SliceDefinition d:defs)
            b.append("  ").append(d).append("\n");
        b.append(super.toString());
        return b.toString();
    }
}
