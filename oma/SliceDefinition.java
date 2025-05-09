package de.kumakyoo.omalibjava;

public class SliceDefinition
{
    public byte type;
    public BoundingBox bounds;
    public String key;
    public String value;

    public SliceDefinition(byte type, BoundingBox bounds, String key, String value)
    {
        this.type = type;
        this.bounds = bounds;
        this.key = key;
        this.value = value;
    }

    public String toString()
    {
        return ((char)type)+", "+bounds+", "+key+", "+value;
    }
}
