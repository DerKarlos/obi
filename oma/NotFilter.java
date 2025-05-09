package de.kumakyoo.omalibjava;

public class NotFilter extends Filter
{
    private Filter f;

    private boolean c,b,s;

    public NotFilter(Filter f)
    {
        this.f = f;
    }

    public boolean needsChunk(byte type, BoundingBox b)
    {
        c = f.needsChunk(type,b);
        return true;
    }

    public boolean needsBlock(String key)
    {
        b = f.needsBlock(key);
        return true;
    }

    public boolean needsSlice(String value)
    {
        s = f.needsSlice(value);
        return true;
    }

    public boolean keep(Element e)
    {
        return !c || !b || !s || !f.keep(e);
    }

    public boolean countable()
    {
        return !c || !b || !s;
    }
}
