package de.kumakyoo.omalibjava;

public class OrFilter extends Filter
{
    private Filter[] fs;

    private boolean[] c;
    private boolean[] b;
    private boolean[] s;

    public OrFilter(Filter... fs)
    {
        this.fs = fs;
        c = new boolean[fs.length];
        b = new boolean[fs.length];
        s = new boolean[fs.length];
    }

    public boolean needsChunk(byte type, BoundingBox b)
    {
        for (int i=0;i<fs.length;i++)
            c[i] = fs[i].needsChunk(type,b);
        for (int i=0;i<fs.length;i++)
            if (c[i]) return true;
        return false;
    }

    public boolean needsBlock(String key)
    {
        for (int i=0;i<fs.length;i++)
            b[i] = c[i] && fs[i].needsBlock(key);
        for (int i=0;i<fs.length;i++)
            if (b[i]) return true;
        return false;
    }

    public boolean needsSlice(String value)
    {
        for (int i=0;i<fs.length;i++)
            s[i] = b[i] && fs[i].needsSlice(value);
        for (int i=0;i<fs.length;i++)
            if (s[i]) return true;
        return false;
    }

    public boolean keep(Element e)
    {
        for (int i=0;i<fs.length;i++)
            if (s[i] && fs[i].keep(e)) return true;
        return false;
    }

    public boolean countable()
    {
        for (int i=0;i<fs.length;i++)
            if (!s[i]) return false;
        for (Filter f: fs)
            if (!f.countable()) return false;
        return true;
    }
}
