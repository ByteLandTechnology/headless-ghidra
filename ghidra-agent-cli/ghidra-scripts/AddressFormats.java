import ghidra.program.model.address.Address;
import ghidra.program.model.address.AddressFactory;

public final class AddressFormats {
    private AddressFormats() {
    }

    public static Address resolveAddress(AddressFactory addressFactory, Object addrValue) {
        String normalized = normalizeAddress(addrValue);
        if (normalized == null) {
            return null;
        }
        return addressFactory.getAddress(normalized);
    }

    public static String normalizeAddress(Object addrValue) {
        return AddressStrings.normalize(addrValue);
    }
}
