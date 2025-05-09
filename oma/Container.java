package de.kumakyoo.omalibjava;

public interface Container
{
    public boolean contains(int lon, int lat);
    public boolean contains(int[] lon, int[] lat);
    public boolean contains(BoundingBox b);
}
