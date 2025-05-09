package de.kumakyoo.omalibjava;

public class VersionFilter extends Filter
{
    private int version;

    public VersionFilter(int version)
    {
        this.version = version;
    }

    public boolean keep(Element e)
    {
        return e.version==version;
    }

    public boolean countable()
    {
        return false;
    }
}
