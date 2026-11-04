"""Read a release bundle one keyed record at a time using the standard library."""
import json


class Cursor:
    """Keep only the current JSON record and a read buffer in memory."""

    def __init__(self, handle):
        self.handle = handle
        self.buffer = ""
        self.decoder = json.JSONDecoder()
        self.eof = False

    def fill(self):
        chunk = self.handle.read(65536)
        self.buffer += chunk
        self.eof = not chunk

    def peek(self):
        self.buffer = self.buffer.lstrip()
        while not self.buffer and not self.eof:
            self.fill()
            self.buffer = self.buffer.lstrip()
        return self.buffer[:1]

    def take(self, expected):
        if self.peek() != expected:
            raise ValueError(f"bundle_expected_{expected}")
        self.buffer = self.buffer[1:]

    def value(self):
        self.peek()
        while True:
            try:
                value, end = self.decoder.raw_decode(self.buffer)
                # A numeric token may continue across the read boundary.
                if end == len(self.buffer) and not self.eof:
                    self.fill()
                    continue
                self.buffer = self.buffer[end:]
                return value
            except json.JSONDecodeError:
                if self.eof:
                    raise ValueError("bundle_invalid_json") from None
                self.fill()

    def keys(self):
        self.take("{")
        if self.peek() == "}":
            self.take("}")
            return
        while True:
            key = self.value()
            if not isinstance(key, str):
                raise ValueError("bundle_nonstring_key")
            self.take(":")
            yield key
            if self.peek() == "}":
                self.take("}")
                return
            self.take(",")


def bundle_records(handle):
    """Yield (section, source identity, object); reject trailing garbage."""
    cursor = Cursor(handle)
    for section in cursor.keys():
        for identity in cursor.keys():
            record = cursor.value()
            if not isinstance(record, dict):
                raise ValueError("bundle_record_not_object")
            yield section, identity, record
    if cursor.peek():
        raise ValueError("bundle_trailing_data")
