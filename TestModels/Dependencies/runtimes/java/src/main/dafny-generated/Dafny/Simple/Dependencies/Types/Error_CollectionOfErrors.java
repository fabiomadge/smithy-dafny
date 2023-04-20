// Class Error_CollectionOfErrors
// Dafny class Error_CollectionOfErrors compiled into Java
package Dafny.Simple.Dependencies.Types;


@SuppressWarnings({"unchecked", "deprecation"})
public class Error_CollectionOfErrors extends Error {
  public dafny.DafnySequence<? extends Error> _list;
  public Error_CollectionOfErrors (dafny.DafnySequence<? extends Error> list) {
    this._list = list;
  }

  @Override
  public boolean equals(Object other) {
    if (this == other) return true;
    if (other == null) return false;
    if (getClass() != other.getClass()) return false;
    Error_CollectionOfErrors o = (Error_CollectionOfErrors)other;
    return true && java.util.Objects.equals(this._list, o._list);
  }
  @Override
  public int hashCode() {
    long hash = 5381;
    hash = ((hash << 5) + hash) + 3;
    hash = ((hash << 5) + hash) + java.util.Objects.hashCode(this._list);
    return (int)hash;
  }

  @Override
  public String toString() {
    StringBuilder s = new StringBuilder();
    s.append("Dafny.Simple.Dependencies.Types_Compile.Error.CollectionOfErrors");
    s.append("(");
    s.append(dafny.Helpers.toString(this._list));
    s.append(")");
    return s.toString();
  }
}
