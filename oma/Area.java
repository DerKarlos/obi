package de.kumakyoo.omalibjava;

import java.io.*;

public class Area extends Element
{
    public int[] lon;
    public int[] lat;

    public int[][] holes_lon;
    public int[][] holes_lat;

    public Area(int[] lon, int[] lat)
    {
        this.lon = lon;
        this.lat = lat;
        holes_lon = new int[0][];
        holes_lat = new int[0][];
    }

    public Area(int[] lon, int[] lat, int[][] holes_lon, int[][] holes_lat)
    {
        this.lon = lon;
        this.lat = lat;
        this.holes_lon = holes_lon;
        this.holes_lat = holes_lat;
    }

    public Area(OmaInputStream in, String key, String value) throws IOException
    {
        super(key,value);
        int count = in.readSmallInt();
        lon = new int[count];
        lat = new int[count];
        for (int i=0;i<count;i++)
        {
            lon[i] = in.readDeltaX();
            lat[i] = in.readDeltaY();
        }

        count = in.readSmallInt();
        holes_lon = new int[count][];
        holes_lat = new int[count][];
        for (int i=0;i<count;i++)
        {
            int holeCount = in.readSmallInt();
            holes_lon[i] = new int[holeCount];
            holes_lat[i] = new int[holeCount];
            for (int j=0;j<holeCount;j++)
            {
                holes_lon[i][j] = in.readDeltaX();
                holes_lat[i][j] = in.readDeltaY();
            }
        }
    }

    public void writeGeo(OmaOutputStream out) throws IOException
    {
        out.writeSmallInt(lon.length);
        for (int k=0;k<lon.length;k++)
        {
            out.writeDeltaX(lon[k]);
            out.writeDeltaY(lat[k]);
        }
        out.writeSmallInt(holes_lon.length);
        for (int k=0;k<holes_lon.length;k++)
        {
            out.writeSmallInt(holes_lon[k].length);
            for (int i=0;i<holes_lon[k].length;i++)
            {
                out.writeDeltaX(holes_lon[k][i]);
                out.writeDeltaY(holes_lat[k][i]);
            }
        }
    }

    public boolean isInside(Container c)
    {
        return c.contains(lon,lat);
    }

    public String toString()
    {
        StringBuffer b = new StringBuffer();
        b.append("area @");
        for (int i=0;i<lon.length;i++)
            b.append(" ").append(lon[i]/1e7).append(",").append(lat[i]/1e7);
        b.append("\n");
        if (holes_lon.length>0)
        {
            b.append("  holes:\n");
            for (int j=0;j<holes_lon.length;j++)
            {
                for (int i=0;i<holes_lon[j].length;i++)
                    b.append(" ").append(holes_lon[j][i]/1e7).append(",").append(holes_lat[j][i]/1e7);
                b.append("\n");
            }
        }
        b.append(super.toString());
        return b.toString();
    }
}
