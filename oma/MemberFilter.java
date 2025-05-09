package de.kumakyoo.omalibjava;

public class MemberFilter extends Filter
{
    Collection c;
    String role;

    public MemberFilter(Collection c)
    {
        this(c,null);
    }

    public MemberFilter(Collection c, String role)
    {
        this.c = c;
        this.role = role;
    }

    public boolean keep(Element e)
    {
        for (Member m:e.members)
            if (m.id==c.id && (role==null || role.equals(m.role)))
                return true;
        return false;
    }
}
