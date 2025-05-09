package de.kumakyoo.omalibjava;

public class ChangesetFilter extends Filter
{
    private long start, end;

    public ChangesetFilter(long start)
    {
        this.start = this.end = start;
    }

    public ChangesetFilter(long start, long end)
    {
        this.start = start;
        this.end = end;
    }

    public boolean keep(Element e)
    {
        return e.changeset>=start && e.changeset<=end;
    }

    public boolean countable()
    {
        return false;
    }
}
