package de.kumakyoo.omalibjava;

public class TimestampFilter extends Filter
{
    private long start, end;

    public TimestampFilter(long start)
    {
        this.start = this.end = start;
    }

    public TimestampFilter(long start, long end)
    {
        this.start = start;
        this.end = end;
    }

    public boolean keep(Element e)
    {
        return e.timestamp>=start && e.timestamp<=end;
    }

    public boolean countable()
    {
        return false;
    }
}
