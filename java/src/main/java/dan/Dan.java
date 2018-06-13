/*
 * This file is part of DAN.
 *
 * DAN is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * DAN is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with DAN.  If not, see <http://www.gnu.org/licenses/>.
 */
package dan;

import com.sun.jna.Pointer;

import java.net.SocketAddress;
import java.nio.ByteBuffer;

public final class Dan implements AutoCloseable {

    private final Pointer dan;

    public Dan(final SocketAddress bindingAddress,
               final SocketAddress connectionAddress,
               final int socketTimeout,
               final int readSize,
               final int writeSize,
               final int packetSize,
               final int packetTime) {

        dan = NativeDanLibrary.dan_create(
                bindingAddress.toString(),
                connectionAddress.toString(),
                socketTimeout,
                readSize,
                writeSize,
                packetSize,
                packetTime);
    }

    @Override
    public void close() {
        NativeDanLibrary.dan_destroy(dan);
    }

    public boolean discoverIp(final ByteBuffer packet) {
        return NativeDanLibrary.dan_discover_ip(dan, packet, packet.capacity());
    }

    public boolean read(final ByteBuffer packet) {
        return NativeDanLibrary.dan_read(dan, packet);
    }

    public boolean readSocket() {
        return NativeDanLibrary.dan_read_socket(dan);
    }

    public boolean write(final ByteBuffer packet) {
        return NativeDanLibrary.dan_write(dan, packet);
    }

    public boolean writeSocket() {
        return NativeDanLibrary.dan_write_socket(dan);
    }
}
