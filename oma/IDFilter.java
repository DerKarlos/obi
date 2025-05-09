package de.kumakyoo.omalibjava;

public class IDFilter extends Filter
{
    private long id;

    public IDFilter(long id)
    {
        this.id = id;
    }

    public boolean keep(Element e)
    {
        return e.id==id;
    }

    public boolean countable()
    {
        return false;
    }
}
