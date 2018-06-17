/*
 * This file is part of DAN.
 *
 * DAN is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * DAN is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with DAN.  If not, see <https://www.gnu.org/licenses/>.
 */
package dan;

import com.sun.jna.Pointer;

import java.net.InetSocketAddress;
import java.nio.ByteBuffer;
import java.util.Objects;

public final class Dan implements AutoCloseable {

    private static String toAddressString(final InetSocketAddress address) {
        // Needed because InetSocketAddress#toString returns "/host:port"
        return address.getHostString() + ":" + address.getPort();
    }

    private final Pointer dan;

    public Dan(final InetSocketAddress binding, final InetSocketAddress connection, final int timeout) {
        dan = NativeDan.dan_create(toAddressString(binding), toAddressString(connection), timeout);

        if (Objects.equals(Pointer.NULL, dan)) { // Rust code failed internally
            throw new IllegalStateException("Native \"dan\" Pointer is NULL");
        }
    }

    @Override
    public void close() {
        NativeDan.dan_destroy(dan);
    }

    public boolean discoverIp(final ByteBuffer packet) {
        return NativeDan.dan_discover_ip(dan, packet, packet.capacity());
    }

    public boolean reading(final int packetSize) {
        return NativeDan.dan_reading(dan, packetSize);
    }

    public boolean read(final ByteBuffer packet) {
        return NativeDan.dan_read(dan, packet, packet.capacity());
    }

    public boolean writing(final int packetTime) {
        return NativeDan.dan_writing(dan, packetTime);
    }

    public boolean write(final ByteBuffer packet) {
        return NativeDan.dan_write(dan, packet, packet.capacity());
    }
}
