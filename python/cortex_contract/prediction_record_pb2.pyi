from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class PredictionRecord(_message.Message):
    __slots__ = ("entity_id", "model_name", "model_version", "schema_version", "event_time_ms", "predict_time_ms", "features", "output", "request_id")
    ENTITY_ID_FIELD_NUMBER: _ClassVar[int]
    MODEL_NAME_FIELD_NUMBER: _ClassVar[int]
    MODEL_VERSION_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_VERSION_FIELD_NUMBER: _ClassVar[int]
    EVENT_TIME_MS_FIELD_NUMBER: _ClassVar[int]
    PREDICT_TIME_MS_FIELD_NUMBER: _ClassVar[int]
    FEATURES_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    entity_id: str
    model_name: str
    model_version: str
    schema_version: int
    event_time_ms: int
    predict_time_ms: int
    features: _containers.RepeatedScalarFieldContainer[float]
    output: _containers.RepeatedScalarFieldContainer[float]
    request_id: str
    def __init__(self, entity_id: _Optional[str] = ..., model_name: _Optional[str] = ..., model_version: _Optional[str] = ..., schema_version: _Optional[int] = ..., event_time_ms: _Optional[int] = ..., predict_time_ms: _Optional[int] = ..., features: _Optional[_Iterable[float]] = ..., output: _Optional[_Iterable[float]] = ..., request_id: _Optional[str] = ...) -> None: ...
