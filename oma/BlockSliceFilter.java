package de.kumakyoo.omalibjava;

public class BlockSliceFilter extends Filter
{
    private String key;
    private String value;

    public BlockSliceFilter(String key, String value)
    {
        if (key==null) key="";
        if (value==null) value="";
        this.key = key;
        this.value = value;
    }

    public boolean needsBlock(String key)
    {
        return this.key.equals(key);
    }

    public boolean needsSlice(String value)
    {
        return this.value.equals(value);
    }
}
