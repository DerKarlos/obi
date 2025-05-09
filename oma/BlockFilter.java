package de.kumakyoo.omalibjava;

public class BlockFilter extends Filter
{
    private String key;

    public BlockFilter(String key)
    {
        if (key==null) key="";
        this.key = key;
    }

    public boolean needsBlock(String key)
    {
        return this.key.equals(key);
    }
}
