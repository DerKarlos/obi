package de.kumakyoo.omalibjava;

public class KeyFilter extends Filter
{
    private String key;

    public KeyFilter(String key)
    {
        if (key==null) key="";
        this.key = key;
    }

    public boolean keep(Element e)
    {
        return e.tags.containsKey(key);
    }

    public boolean countable()
    {
        return false;
    }
}
