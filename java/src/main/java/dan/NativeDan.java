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
 * along with DAN.  If not, see <https://www.gnu.org/licenses/>.
 */
package dan;

import com.sun.jna.Native;
import com.sun.jna.Pointer;

import java.nio.ByteBuffer;

@SuppressWarnings({"NativeMethod", "StaticMethodOnlyUsedInOneClass"})
final class NativeDan {

    static native Pointer dan_create(String bindingAddress, String connectionAddress, int socketTimeout);
    static native boolean dan_destroy(Pointer dan);
    static native boolean dan_discover_ip(Pointer dan, ByteBuffer packet, int packetSize);
    static native boolean dan_reading(Pointer dan, int packetSize);
    static native boolean dan_read(Pointer dan, ByteBuffer packet, int packetSize);
    static native boolean dan_writing(Pointer dan, int packetTime);
    static native boolean dan_write(Pointer dan, ByteBuffer packet, int packetSize);

    static {
        // "Name" varies by OS
        Native.register("dan");
    }

    private NativeDan() {}
}
