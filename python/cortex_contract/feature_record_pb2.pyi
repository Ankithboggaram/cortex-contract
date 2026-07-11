from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class FeatureRecord(_message.Message):
    __slots__ = ("schema_version", "event_time_ms", "features")
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    EVENT_TIME_MS_FIELD_NUMBER: _ClassVar[int]
    FEATURES_FIELD_NUMBER: _ClassVar[int]
    schema_version: int
    event_time_ms: int
    features: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, schema_version: _Optional[int] = ..., event_time_ms: _Optional[int] = ..., features: _Optional[_Iterable[float]] = ...) -> None: ...
