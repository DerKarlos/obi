package de.kumakyoo.omalibjava;

import java.io.*;
import java.util.*;

abstract public class Element
{
    public long id;
    public int version;
    public long timestamp;
    public long changeset;
    public int uid;
    public String user;

    public String key;
    public String value;

    public Map<String, String> tags;
    public Member[] members;

    protected Element()
    {
        this.id = Long.MIN_VALUE;
        this.version = Integer.MIN_VALUE;
        this.timestamp = Long.MIN_VALUE;
        this.changeset = Long.MIN_VALUE;
        this.uid = Integer.MIN_VALUE;
        this.user = null;
        this.tags = null;
        key = null;
        value = null;
    }

    protected Element(String key, String value)
    {
        this();
        this.key = key;
        this.value = value;
    }

    public void write(OmaOutputStream out, int features) throws IOException
    {
        writeGeo(out);
        writeTags(out);
        writeMembers(out);
        writeMetaData(out,features);
    }

    abstract public void writeGeo(OmaOutputStream out) throws IOException;

    public void writeTags(OmaOutputStream out) throws IOException
    {
        out.writeSmallInt(tags.size());
        for (String key:tags.keySet())
        {
            out.writeString(key);
            out.writeString(tags.get(key));
        }
    }

    public void writeMembers(OmaOutputStream out) throws IOException
    {
        out.writeSmallInt(members.length);
        for (Member m:members)
        {
            out.writeLong(m.id);
            out.writeString(m.role);
            out.writeSmallInt(m.nr);
        }
    }

    public void writeMetaData(OmaOutputStream out, int features) throws IOException
    {
        if ((features&2)!=0 || this instanceof Collection)
            out.writeLong(id);
        if ((features&4)!=0)
            out.writeSmallInt(version);
        if ((features&8)!=0)
            out.writeLong(timestamp);
        if ((features&16)!=0)
            out.writeLong(changeset);
        if ((features&32)!=0)
        {
            out.writeInt(uid);
            out.writeString(user);
        }
    }

    void readTags(OmaInputStream in) throws IOException
    {
        tags = new HashMap<>();
        int count = in.readSmallInt();
        for (int i=0;i<count;i++)
            tags.put(in.readString(),in.readString());
    }

    void readMembers(OmaInputStream in) throws IOException
    {
        int count = in.readSmallInt();
        members = new Member[count];
        for (int i=0;i<count;i++)
            members[i] = new Member(in.readLong(),in.readString(),in.readSmallInt());
    }

    void readMeta(OmaInputStream in, int features) throws IOException
    {
        if ((features&2)!=0)
            id = in.readLong();
        if ((features&4)!=0)
            version = in.readSmallInt();
        if ((features&8)!=0)
            timestamp = in.readLong();
        if ((features&16)!=0)
            changeset = in.readLong();
        if ((features&32)!=0)
        {
            uid = in.readInt();
            user = in.readString();
        }
    }

    abstract public boolean isInside(Container c);

    public String toString()
    {
        StringBuffer b = new StringBuffer();
        b.append(key).append(" (").append(value).append(")\n");
        if (id!=Long.MIN_VALUE)
            b.append("  id=").append(id).append("\n");
        if (version!=Integer.MIN_VALUE)
            b.append("  version=").append(version).append("\n");
        if (timestamp!=Long.MIN_VALUE)
            b.append("  timestamp=").append(timestamp).append("\n");
        if (changeset!=Long.MIN_VALUE)
            b.append("  changeset=").append(changeset).append("\n");
        if (uid!=Integer.MIN_VALUE)
            b.append("  uid=").append(uid).append("\n");
        if (user!=null)
            b.append("  user=").append(user).append("\n");

        if (tags!=null)
            for (String tag:tags.keySet())
                b.append("    ").append(tag).append(" = ").append(tags.get(tag)).append("\n");

        for (Member m:members)
            b.append("    ").append(m.id).append(" ").append(m.role).append(" ").append(m.nr).append("\n");

        return b.toString();
    }
}
