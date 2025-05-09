package de.kumakyoo.omalibjava;

public class TypeFilter extends Filter
{
    private byte[] types;

    public TypeFilter(byte type)
    {
        types = new byte[1];
        types[0] = type;
    }

    public TypeFilter(char type)
    {
        types = new byte[1];
        types[0] = (byte)type;
    }

    public TypeFilter(byte[] types)
    {
        this.types = types;
    }

    public TypeFilter(String types)
    {
        this.types = new byte[types.length()];
        for (int i=0;i<types.length();i++)
            this.types[i] = (byte)types.charAt(i);
    }

    public boolean needsChunk(byte type, BoundingBox b)
    {
        for (byte t:types)
            if (t==type)
                return true;
        return false;
    }
}
