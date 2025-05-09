package de.kumakyoo.omalibjava;

public class Filter
{
    public boolean needsChunk(byte type, BoundingBox b)
    {
        return true;
    }

    public boolean needsBlock(String key)
    {
        return true;
    }

    public boolean needsSlice(String value)
    {
        return true;
    }

    public boolean keep(Element e)
    {
        return true;
    }

    public boolean countable()
    {
        return true;
    }
}
