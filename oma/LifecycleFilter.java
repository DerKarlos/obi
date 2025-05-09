package de.kumakyoo.omalibjava;

public class LifecycleFilter extends Filter
{
    private String lifecycle;

    public LifecycleFilter()
    {
        lifecycle = null;
    }

    public LifecycleFilter(String lifecycle)
    {
        this.lifecycle = lifecycle;
        if ("".equals(lifecycle))
            this.lifecycle = null;
    }

    public boolean keep(Element e)
    {
        if (lifecycle==null)
            return !e.tags.containsKey("lifecycle");
        return lifecycle.equals(e.tags.get("lifecycle"));
    }

    public boolean countable()
    {
        return false;
    }
}
