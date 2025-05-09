package de.kumakyoo.omalibjava;

public class TagFilter extends Filter
{
    private String key, value;

    public TagFilter(String key, String value)
    {
        if (key==null) key="";
        if (value==null) value="";
        this.key = key;
        this.value = value;
    }

    public boolean keep(Element e)
    {
        return value.equals(e.tags.get(key));
    }

    public boolean countable()
    {
        return false;
    }
}
