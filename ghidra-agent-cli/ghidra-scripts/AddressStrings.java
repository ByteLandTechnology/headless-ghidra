import java.math.BigInteger;

public final class AddressStrings {
    private AddressStrings() {
    }

    public static String normalize(Object addrValue) {
        if (addrValue == null) {
            return null;
        }
        if (addrValue instanceof Byte || addrValue instanceof Short || addrValue instanceof Integer
            || addrValue instanceof Long || addrValue instanceof BigInteger) {
            return new BigInteger(String.valueOf(addrValue)).toString(16);
        }
        return String.valueOf(addrValue);
    }
}
