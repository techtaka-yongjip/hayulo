from __future__ import annotations

import json
import re
import shutil
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .diagnostics import Diagnostic, HayuloError, HayuloSyntaxError

API_METHODS = {"GET", "POST", "PUT", "PATCH", "DELETE"}
BUILTIN_TYPES = {"Text", "Int", "Float", "Bool", "Time", "Email", "Status", "Any"}


class HayuloApiError(HayuloError):
    pass


@dataclass
class ApiField:
    name: str
    type_name: str
    line: int
    default: str | None = None
    constraints: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {"name": self.name, "type": self.type_name, "line": self.line}
        if self.default is not None:
            data["default"] = self.default
        if self.constraints:
            data["constraints"] = self.constraints
        return data


@dataclass
class ApiRecord:
    name: str
    fields: list[ApiField]
    line: int

    def field_names(self) -> set[str]:
        return {f.name for f in self.fields}

    def to_dict(self) -> dict[str, Any]:
        return {"name": self.name, "line": self.line, "fields": [f.to_dict() for f in self.fields]}


@dataclass
class ApiRoute:
    method: str
    path: str
    response_type: str
    line: int
    body_name: str | None = None
    body_type: str | None = None
    auth_name: str | None = None
    auth_type: str | None = None

    def to_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {"method": self.method, "path": self.path, "response_type": self.response_type, "line": self.line}
        if self.body_type:
            data["body"] = {"name": self.body_name, "type": self.body_type}
        if self.auth_type:
            data["auth"] = {"name": self.auth_name, "type": self.auth_type}
        return data


@dataclass
class ApiDatabase:
    kind: str
    value: str
    line: int

    def to_dict(self) -> dict[str, Any]:
        return {"kind": self.kind, "value": self.value, "line": self.line}


@dataclass
class ApiOpenApi:
    title: str = "Hayulo API"
    version: str = "0.1.0"

    def to_dict(self) -> dict[str, str]:
        return {"title": self.title, "version": self.version}


@dataclass
class ApiSpec:
    module: str | None
    app_name: str
    database: ApiDatabase | None
    openapi: ApiOpenApi
    records: dict[str, ApiRecord]
    routes: list[ApiRoute]

    def to_dict(self) -> dict[str, Any]:
        return {
            "module": self.module,
            "app": self.app_name,
            "database": self.database.to_dict() if self.database else None,
            "openapi": self.openapi.to_dict(),
            "records": [r.to_dict() for r in self.records.values()],
            "routes": [r.to_dict() for r in self.routes],
        }


@dataclass
class GeneratedFile:
    path: Path
    description: str

    def to_dict(self) -> dict[str, str]:
        return {"path": str(self.path), "description": self.description}


def looks_like_api_source(source: str) -> bool:
    return bool(re.search(r"(^|\n)\s*app\s+[A-Za-z_]\w*\s*\{", source))


def parse_api_source(source: str, filename: str | None = None) -> ApiSpec:
    spec = ApiSourceParser(source, filename).parse()
    check_api_spec(spec, filename)
    return spec


class ApiSourceParser:
    def __init__(self, source: str, filename: str | None):
        self.lines = source.splitlines()
        self.filename = filename
        self.module: str | None = None
        self.app_name: str | None = None
        self.database: ApiDatabase | None = None
        self.openapi = ApiOpenApi()
        self.records: dict[str, ApiRecord] = {}
        self.routes: list[ApiRoute] = []

    def parse(self) -> ApiSpec:
        self._parse_lines(self.lines, base_line=1, in_app=False)
        if not self.app_name:
            self._err("missing_app", "Hayulo API source must declare an app block.", 1, ["Add: app TodoApi { ... }"])
        return ApiSpec(self.module, self.app_name, self.database, self.openapi, self.records, self.routes)  # type: ignore[arg-type]

    def _parse_lines(self, lines: list[str], *, base_line: int, in_app: bool) -> None:
        i = 0
        while i < len(lines):
            raw = lines[i]
            line_no = base_line + i
            line = strip_comment(raw).strip()
            if not line:
                i += 1
                continue
            m = re.match(r"module\s+([A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*)\s*$", line)
            if m and not in_app:
                self.module = m.group(1)
                i += 1
                continue
            if re.match(r"intent\s*\{", line):
                _, i = collect_block(lines, i, self.filename, "intent")
                continue
            m = re.match(r"app\s+([A-Za-z_]\w*)\s*\{", line)
            if m and not in_app:
                self.app_name = m.group(1)
                block, i = collect_block(lines, i, self.filename, "app")
                self._parse_lines(block, base_line=line_no + 1, in_app=True)
                continue
            if re.match(r"database\s+", line) and in_app:
                self.database = self._database(line, line_no)
                i += 1
                continue
            if re.match(r"openapi\s*\{", line) and in_app:
                block, i = collect_block(lines, i, self.filename, "openapi")
                self.openapi = self._openapi(block, line_no + 1)
                continue
            if re.match(r"type\s+", line):
                block, i = collect_block(lines, i, self.filename, "record")
                record = self._record(line, block, line_no)
                if record.name in self.records:
                    self._err("duplicate_record", f"Record {record.name!r} is already defined.", line_no, ["Rename or remove the duplicate record."])
                self.records[record.name] = record
                continue
            if re.match(r"route\s+", line) and in_app:
                _, i = collect_block(lines, i, self.filename, "route")
                self.routes.append(self._route(line, line_no))
                continue
            if re.match(r"test\s+", line):
                _, i = collect_block(lines, i, self.filename, "test")
                continue
            allowed = "module, intent, app, or type" if not in_app else "database, openapi, type, route, or intent"
            self._err("unexpected_api_line", f"Unexpected Hayulo API line: {line!r}.", line_no, [f"Expected {allowed}."])

    def _database(self, line: str, line_no: int) -> ApiDatabase:
        m = re.match(r"database\s+([A-Za-z_]\w*)\s+(?:\"([^\"]+)\"|([^\s]+))\s*$", line)
        if not m:
            self._err("invalid_database", "Invalid database declaration.", line_no, ["Use: database sqlite \"todo.db\"."])
        return ApiDatabase(m.group(1), m.group(2) or m.group(3), line_no)

    def _openapi(self, block: list[str], base_line: int) -> ApiOpenApi:
        result = ApiOpenApi()
        for offset, raw in enumerate(block):
            line = strip_comment(raw).strip().rstrip(",")
            if not line:
                continue
            m = re.match(r"([A-Za-z_]\w*)\s*:\s*\"([^\"]*)\"\s*$", line)
            if not m:
                self._err("invalid_openapi_field", f"Invalid openapi line: {line!r}.", base_line + offset, ["Use title: \"...\" or version: \"...\"."])
            if m.group(1) == "title":
                result.title = m.group(2)
            elif m.group(1) == "version":
                result.version = m.group(2)
            else:
                self._err("unknown_openapi_field", f"Unknown openapi field {m.group(1)!r}.", base_line + offset, ["Supported fields are title and version."])
        return result

    def _record(self, header: str, block: list[str], line_no: int) -> ApiRecord:
        m = re.match(r"type\s+([A-Za-z_]\w*)\s*=\s*record\s*\{", header)
        if not m:
            self._err("invalid_record_header", "Invalid record declaration.", line_no, ["Use: type Todo = record { ... }."])
        fields: list[ApiField] = []
        for offset, raw in enumerate(block):
            line = strip_comment(raw).strip().rstrip(",")
            if not line:
                continue
            fm = re.match(r"([A-Za-z_]\w*)\s*:\s*(.+)$", line)
            if not fm:
                self._err("invalid_field", f"Invalid field line: {line!r}.", line_no + 1 + offset, ["Use: name: Type min 1 max 200."])
            fields.append(parse_field(fm.group(1), fm.group(2), line_no + 1 + offset, self.filename))
        if not fields:
            self._err("empty_record", f"Record {m.group(1)} has no fields.", line_no, ["Add at least one field."])
        return ApiRecord(m.group(1), fields, line_no)

    def _route(self, header: str, line_no: int) -> ApiRoute:
        head = header.rsplit("{", 1)[0].strip()
        m = re.match(r"route\s+(GET|POST|PUT|PATCH|DELETE)\s+\"([^\"]+)\"\s*(.*?)\s*->\s*([A-Za-z_]\w*(?:<[^>]+>)?)\s*$", head)
        if not m:
            self._err("invalid_route", "Invalid route declaration.", line_no, ["Use: route GET \"/todos\" -> List<Todo> { ... }."])
        clauses = m.group(3)
        body_name = body_type = auth_name = auth_type = None
        bm = re.search(r"\bbody\s+([A-Za-z_]\w*)\s*:\s*([A-Za-z_]\w*(?:<[^>]+>)?)", clauses)
        if bm:
            body_name, body_type = bm.group(1), compact_type(bm.group(2))
        am = re.search(r"\bauth\s+([A-Za-z_]\w*)\s*:\s*([A-Za-z_]\w*(?:<[^>]+>)?)", clauses)
        if am:
            auth_name, auth_type = am.group(1), compact_type(am.group(2))
        return ApiRoute(m.group(1), m.group(2), compact_type(m.group(4)), line_no, body_name, body_type, auth_name, auth_type)

    def _err(self, code: str, message: str, line: int, suggestions: list[str]) -> None:
        raise HayuloSyntaxError(Diagnostic(code=code, message=message, file=self.filename, line=line, suggestions=suggestions))


def strip_comment(line: str) -> str:
    in_string = False
    escaped = False
    i = 0
    while i < len(line):
        ch = line[i]
        if escaped:
            escaped = False
        elif ch == "\\":
            escaped = True
        elif ch == '"':
            in_string = not in_string
        elif not in_string and line[i : i + 2] == "//":
            return line[:i]
        i += 1
    return line


def count_braces(line: str) -> int:
    text = strip_comment(line)
    in_string = False
    escaped = False
    count = 0
    for ch in text:
        if escaped:
            escaped = False
        elif ch == "\\":
            escaped = True
        elif ch == '"':
            in_string = not in_string
        elif not in_string and ch == "{":
            count += 1
        elif not in_string and ch == "}":
            count -= 1
    return count


def collect_block(lines: list[str], start: int, filename: str | None, label: str) -> tuple[list[str], int]:
    if "{" not in strip_comment(lines[start]):
        raise HayuloSyntaxError(Diagnostic(code="missing_block", message=f"Expected '{{' for {label} block.", file=filename, line=start + 1))
    depth = count_braces(lines[start])
    block: list[str] = []
    i = start + 1
    while i < len(lines):
        delta = count_braces(lines[i])
        if depth + delta <= 0:
            before = lines[i].rsplit("}", 1)[0].strip()
            if before:
                block.append(before)
            return block, i + 1
        block.append(lines[i])
        depth += delta
        i += 1
    raise HayuloSyntaxError(Diagnostic(code="unterminated_block", message=f"Unterminated {label} block.", file=filename, line=start + 1, suggestions=["Add a closing '}'."]))


def parse_field(name: str, rhs: str, line: int, filename: str | None) -> ApiField:
    default: str | None = None
    if "=" in rhs:
        rhs, default = rhs.split("=", 1)
        default = default.strip()
    parts = rhs.strip().split()
    if not parts:
        raise HayuloSyntaxError(Diagnostic(code="missing_field_type", message=f"Field {name!r} is missing a type.", file=filename, line=line))
    type_name = compact_type(parts[0])
    constraints: dict[str, Any] = {}
    i = 1
    while i < len(parts):
        key = parts[i]
        if key in {"unique", "private"}:
            constraints[key] = True
            i += 1
        elif key in {"min", "max"}:
            if i + 1 >= len(parts):
                raise HayuloSyntaxError(Diagnostic(code="missing_constraint_value", message=f"Constraint {key!r} on {name!r} needs a value.", file=filename, line=line))
            value = parts[i + 1]
            try:
                constraints[key] = float(value) if "." in value else int(value)
            except ValueError as exc:
                raise HayuloSyntaxError(Diagnostic(code="invalid_constraint_value", message=f"Constraint {key!r} on {name!r} must be numeric.", file=filename, line=line)) from exc
            i += 2
        else:
            raise HayuloSyntaxError(Diagnostic(code="unknown_field_constraint", message=f"Unknown field constraint {key!r} on {name!r}.", file=filename, line=line, suggestions=["Supported constraints are min, max, unique, and private."]))
    return ApiField(name, type_name, line, default, constraints)


def compact_type(value: str) -> str:
    return re.sub(r"\s+", "", value.strip())


def split_generic_type(type_name: str) -> tuple[str, str | None]:
    m = re.fullmatch(r"([A-Za-z_]\w*)<(.+)>", type_name)
    return (m.group(1), m.group(2).strip()) if m else (type_name, None)


def validate_type(type_name: str, records: dict[str, ApiRecord], filename: str | None, line: int | None, context: str) -> None:
    base, inner = split_generic_type(type_name)
    if inner is not None:
        if base not in {"List", "Id"}:
            raise HayuloApiError(Diagnostic(code="unsupported_generic_type", message=f"Unsupported generic type {base!r} in {context}.", file=filename, line=line, details={"type": type_name}, suggestions=["Supported generics are List<T> and Id<T>."]))
        validate_type(inner, records, filename, line, context)
        return
    if type_name in BUILTIN_TYPES or type_name in records:
        return
    raise HayuloApiError(Diagnostic(code="unknown_type", message=f"Unknown type {type_name!r} in {context}.", file=filename, line=line, details={"type": type_name}, suggestions=["Define this record or use Text, Int, Bool, Time, Email, Status."]))


def response_record_name(response_type: str) -> str | None:
    if response_type == "Status":
        return None
    base, inner = split_generic_type(response_type)
    return inner if base == "List" and inner else response_type


def check_api_spec(spec: ApiSpec, filename: str | None = None) -> None:
    if not spec.records:
        raise HayuloApiError(Diagnostic(code="api_without_records", message="API app must define at least one record type.", file=filename))
    if not spec.routes:
        raise HayuloApiError(Diagnostic(code="api_without_routes", message="API app must define at least one route.", file=filename))
    for record in spec.records.values():
        seen: set[str] = set()
        for field in record.fields:
            if field.name in seen:
                raise HayuloApiError(Diagnostic(code="duplicate_field", message=f"Record {record.name} repeats field {field.name!r}.", file=filename, line=field.line))
            seen.add(field.name)
            validate_type(field.type_name, spec.records, filename, field.line, f"field {record.name}.{field.name}")
            if "min" in field.constraints and "max" in field.constraints and field.constraints["min"] > field.constraints["max"]:
                raise HayuloApiError(Diagnostic(code="invalid_constraint_range", message=f"Field {field.name!r} has min greater than max.", file=filename, line=field.line))
    for route in spec.routes:
        if route.response_type != "Status":
            validate_type(route.response_type, spec.records, filename, route.line, f"route {route.method} {route.path}")
        if route.body_type:
            validate_type(route.body_type, spec.records, filename, route.line, f"body of route {route.method} {route.path}")
        if route.method == "POST" and not route.body_type:
            raise HayuloApiError(Diagnostic(code="post_without_body", message=f"POST route {route.path} must declare a body input type.", file=filename, line=route.line, suggestions=["Add: body input: CreateTodo."]))
        response_record = response_record_name(route.response_type)
        if response_record and route.body_type and route.body_type in spec.records and response_record in spec.records:
            extra = sorted(spec.records[route.body_type].field_names() - spec.records[response_record].field_names())
            if extra:
                raise HayuloApiError(Diagnostic(code="body_field_not_in_response_record", message=f"Body type {route.body_type} contains fields missing from {response_record}: {', '.join(extra)}.", file=filename, line=route.line, details={"unknown_fields": extra}))


def singularize(value: str) -> str:
    if value.endswith("ies"):
        return value[:-3] + "y"
    if value.endswith("s"):
        return value[:-1]
    return value


def infer_route_record(route: ApiRoute, records: dict[str, ApiRecord]) -> str | None:
    response = response_record_name(route.response_type)
    if response in records:
        return response
    first = singularize(route.path.strip("/").split("/")[0])
    for name in records:
        if name.lower() == first.lower():
            return name
    return None


def infer_action(route: ApiRoute) -> str:
    if route.method == "GET" and route.response_type.startswith("List<"):
        return "list"
    if route.method == "GET":
        return "get"
    if route.method == "POST":
        return "create"
    if route.method == "PATCH" and route.path.rstrip("/").endswith("/done"):
        return "mark_done"
    if route.method == "PATCH":
        return "update"
    if route.method == "DELETE":
        return "delete"
    return "not_implemented"


def openapi_type(type_name: str) -> dict[str, Any]:
    base, inner = split_generic_type(type_name)
    if base == "List" and inner:
        return {"type": "array", "items": openapi_type(inner)}
    if base == "Id" and inner:
        return {"type": "integer"}
    if type_name == "Text":
        return {"type": "string"}
    if type_name == "Email":
        return {"type": "string", "format": "email"}
    if type_name == "Int":
        return {"type": "integer"}
    if type_name == "Float":
        return {"type": "number"}
    if type_name == "Bool":
        return {"type": "boolean"}
    if type_name == "Time":
        return {"type": "string", "format": "date-time"}
    if type_name == "Status":
        return {"type": "object"}
    return {"$ref": f"#/components/schemas/{type_name}"}


def build_openapi(spec: ApiSpec) -> dict[str, Any]:
    schemas: dict[str, Any] = {}
    for record in spec.records.values():
        props: dict[str, Any] = {}
        required: list[str] = []
        for field in record.fields:
            if field.constraints.get("private"):
                continue
            schema = openapi_type(field.type_name)
            if "min" in field.constraints:
                schema["minLength" if schema.get("type") == "string" else "minimum"] = field.constraints["min"]
            if "max" in field.constraints:
                schema["maxLength" if schema.get("type") == "string" else "maximum"] = field.constraints["max"]
            props[field.name] = schema
            if field.default is None and not field.type_name.startswith("Id<"):
                required.append(field.name)
        schemas[record.name] = {"type": "object", "additionalProperties": False, "properties": props, "required": required}
    schemas["ErrorResponse"] = {
        "type": "object",
        "additionalProperties": False,
        "properties": {
            "error": {
                "type": "object",
                "additionalProperties": True,
                "properties": {
                    "code": {"type": "string"},
                    "message": {"type": "string"},
                    "details": {},
                },
                "required": ["code", "message"],
            }
        },
        "required": ["error"],
    }
    paths: dict[str, Any] = {
        "/health": {
            "get": {
                "operationId": "get_health",
                "summary": "Check API health",
                "responses": {
                    "200": {
                        "description": "API is healthy",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {"status": {"type": "string"}, "app": {"type": "string"}},
                                    "required": ["status", "app"],
                                }
                            }
                        },
                    }
                },
            }
        },
        "/openapi.json": {
            "get": {
                "operationId": "get_openapi_json",
                "summary": "Return this OpenAPI document",
                "responses": {"200": {"description": "OpenAPI document"}},
            }
        },
    }
    for route in spec.routes:
        success_status = success_status_code(route)
        op: dict[str, Any] = {
            "operationId": operation_id(route),
            "summary": route_summary(route, spec.records),
            "parameters": path_parameters(route),
            "responses": {
                success_status: response_for_status(success_status, route.response_type),
                "400": error_response("Bad request or validation failure"),
                "404": error_response("Route or resource not found"),
            },
        }
        if route.method == "DELETE":
            op["responses"] = {"204": {"description": "Deleted"}, "404": error_response("Resource not found")}
        if route.body_type:
            op["requestBody"] = {"required": True, "content": {"application/json": {"schema": openapi_type(route.body_type)}}}
        paths.setdefault(route.path, {})[route.method.lower()] = op
    return {"openapi": "3.1.0", "info": spec.openapi.to_dict(), "paths": paths, "components": {"schemas": schemas}}


def success_status_code(route: ApiRoute) -> str:
    if route.method == "POST":
        return "201"
    if route.method == "DELETE":
        return "204"
    return "200"


def response_for_status(status: str, type_name: str) -> dict[str, Any]:
    if status == "204":
        return {"description": "No content"}
    description = "Created" if status == "201" else "OK"
    return {"description": description, "content": {"application/json": {"schema": openapi_type(type_name)}}}


def error_response(description: str) -> dict[str, Any]:
    return {"description": description, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/ErrorResponse"}}}}


def path_parameters(route: ApiRoute) -> list[dict[str, Any]]:
    names = re.findall(r"\{([A-Za-z_]\w*)\}", route.path)
    return [
        {
            "name": name,
            "in": "path",
            "required": True,
            "schema": {"type": "integer"} if name == "id" else {"type": "string"},
        }
        for name in names
    ]


def route_summary(route: ApiRoute, records: dict[str, ApiRecord]) -> str:
    action = infer_action(route).replace("_", " ")
    record = infer_route_record(route, records)
    if record:
        return f"{action.title()} {record}"
    return f"{action.title()} resource"


def operation_id(route: ApiRoute) -> str:
    p = route.path.strip("/").replace("{", "by_").replace("}", "").replace("/", "_").replace("-", "_")
    return f"{route.method.lower()}_{p or 'root'}"


def generate_api(spec: ApiSpec, out_dir: Path, *, clean: bool = True) -> list[GeneratedFile]:
    if clean and out_dir.exists():
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    routes = [{**route.to_dict(), "action": infer_action(route), "record": infer_route_record(route, spec.records)} for route in spec.routes]
    ir = spec.to_dict()
    ir["routes"] = routes
    files = [
        write_file(out_dir / "hayulo.ir.json", json.dumps(ir, indent=2, sort_keys=True) + "\n", "Hayulo API intermediate representation"),
        write_file(out_dir / "openapi.json", json.dumps(build_openapi(spec), indent=2, sort_keys=True) + "\n", "OpenAPI 3.1 specification"),
        write_file(out_dir / "package.json", generated_package_json(spec), "Node package manifest"),
        write_file(out_dir / "server.mjs", generated_server(spec, routes), "Runnable REST API server"),
        write_file(out_dir / "smoke_test.mjs", generated_smoke_test(spec, routes), "Generated smoke test"),
        write_file(out_dir / "README.md", generated_readme(spec), "Generated API instructions"),
    ]
    return files


def write_file(path: Path, content: str, description: str) -> GeneratedFile:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return GeneratedFile(path, description)


def generated_package_json(spec: ApiSpec) -> str:
    name = re.sub(r"[^a-z0-9-]+", "-", spec.app_name.lower()).strip("-") or "hayulo-api"
    return json.dumps({"name": name, "version": spec.openapi.version, "private": True, "type": "module", "scripts": {"start": "node server.mjs", "dev": "node server.mjs", "test": "node smoke_test.mjs"}}, indent=2, sort_keys=True) + "\n"


SERVER_TEMPLATE = r'''// Generated by Hayulo. Edit the .hayulo source, not this file.
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const META = @@META@@;
const OPENAPI = @@OPENAPI@@;
const DB_FILE = path.join(__dirname, `${META.database.value || META.app}.json`);
function emptyStore(){const data={},nextIds={};for(const n of Object.keys(META.records)){data[n]=[];nextIds[n]=1;}return{data,nextIds};}
function loadStore(){if(!fs.existsSync(DB_FILE))return emptyStore();try{const p=JSON.parse(fs.readFileSync(DB_FILE,'utf8'));const e=emptyStore();return{data:{...e.data,...(p.data||{})},nextIds:{...e.nextIds,...(p.nextIds||{})}};}catch{return emptyStore();}}
let store=loadStore();
function save(){fs.writeFileSync(DB_FILE,JSON.stringify(store,null,2)+'\n');}
function send(res,status,body){res.statusCode=status;res.setHeader('Access-Control-Allow-Origin','*');res.setHeader('Access-Control-Allow-Methods','GET,POST,PUT,PATCH,DELETE,OPTIONS');res.setHeader('Access-Control-Allow-Headers','content-type,authorization');if(status===204){res.end();return;}res.setHeader('content-type','application/json; charset=utf-8');res.end(JSON.stringify(body,null,2));}
function error(res,status,code,message,details){send(res,status,{error:{code,message,...(details?{details}:{})}});}
function splitPath(v){return v.replace(/^\/+|\/+$/g,'').split('/').filter(Boolean);}
function matchRoute(pattern,actual){const p=splitPath(pattern),a=splitPath(actual);if(p.length!==a.length)return null;const params={};for(let i=0;i<p.length;i++){if(p[i].startsWith('{')&&p[i].endsWith('}'))params[p[i].slice(1,-1)]=decodeURIComponent(a[i]);else if(p[i]!==a[i])return null;}return params;}
async function readBody(req){const chunks=[];for await(const c of req)chunks.push(c);const text=Buffer.concat(chunks).toString('utf8').trim();if(!text)return{};try{return JSON.parse(text);}catch{const e=new Error('Request body must be valid JSON.');e.status=400;e.code='invalid_json';throw e;}}
function typeCheck(v,t){if(t.startsWith('Id<'))return Number.isInteger(v);if(['Text','Email','Time'].includes(t))return typeof v==='string';if(t==='Int')return Number.isInteger(v);if(t==='Float')return typeof v==='number'&&Number.isFinite(v);if(t==='Bool')return typeof v==='boolean';return true;}
function validateBody(typeName,input){const record=META.records[typeName];if(!record)return{ok:false,errors:[`Unknown body type ${typeName}.`]};const value={},errors=[];for(const f of record.fields){const present=Object.prototype.hasOwnProperty.call(input,f.name);if(!present){if(f.default===undefined||f.default===null)errors.push(`Missing required field ${f.name}.`);continue;}const raw=input[f.name];if(!typeCheck(raw,f.type)){errors.push(`Field ${f.name} must be ${f.type}.`);continue;}if(f.constraints?.min!==undefined){if(typeof raw==='string'&&raw.length<f.constraints.min)errors.push(`Field ${f.name} is shorter than ${f.constraints.min}.`);if(typeof raw==='number'&&raw<f.constraints.min)errors.push(`Field ${f.name} is less than ${f.constraints.min}.`);}if(f.constraints?.max!==undefined){if(typeof raw==='string'&&raw.length>f.constraints.max)errors.push(`Field ${f.name} is longer than ${f.constraints.max}.`);if(typeof raw==='number'&&raw>f.constraints.max)errors.push(`Field ${f.name} is greater than ${f.constraints.max}.`);}value[f.name]=raw;}return errors.length?{ok:false,errors}:{ok:true,value};}
function defaultValue(f,recordName){if(f.type.startsWith('Id<')){const id=store.nextIds[recordName]||1;store.nextIds[recordName]=id+1;return id;}if(f.default==='now()'||f.type==='Time')return new Date().toISOString();if(f.default==='false')return false;if(f.default==='true')return true;if(f.default!==undefined&&f.default!==null){if(/^-?\d+$/.test(f.default))return Number.parseInt(f.default,10);if(/^-?\d+\.\d+$/.test(f.default))return Number.parseFloat(f.default);return String(f.default).replace(/^"|"$/g,'');}return null;}
function createRecord(recordName,input){const record=META.records[recordName],item={};for(const f of record.fields){item[f.name]=Object.prototype.hasOwnProperty.call(input,f.name)?input[f.name]:defaultValue(f,recordName);}store.data[recordName].push(item);save();return item;}
function findById(recordName,id){const n=Number.parseInt(id,10);return store.data[recordName].find(x=>x.id===n);}
async function handle(route,params,req,res){const recordName=route.record;if(route.action==='list')return send(res,200,store.data[recordName]||[]);if(route.action==='get'){const item=findById(recordName,params.id);return item?send(res,200,item):error(res,404,'not_found',`${recordName} not found.`);}if(route.action==='create'){const body=await readBody(req),v=validateBody(route.body.type,body);return v.ok?send(res,201,createRecord(recordName,v.value)):error(res,400,'validation_failed','Request body failed validation.',v.errors);}if(route.action==='mark_done'){const item=findById(recordName,params.id);if(!item)return error(res,404,'not_found',`${recordName} not found.`);item.done=true;save();return send(res,200,item);}if(route.action==='update'){const item=findById(recordName,params.id);if(!item)return error(res,404,'not_found',`${recordName} not found.`);const body=await readBody(req),v=validateBody(route.body.type,body);if(!v.ok)return error(res,400,'validation_failed','Request body failed validation.',v.errors);Object.assign(item,v.value);save();return send(res,200,item);}if(route.action==='delete'){const n=Number.parseInt(params.id,10),before=store.data[recordName].length;store.data[recordName]=store.data[recordName].filter(x=>x.id!==n);if(store.data[recordName].length===before)return error(res,404,'not_found',`${recordName} not found.`);save();return send(res,204,null);}return error(res,501,'not_implemented',`Route action ${route.action} is not implemented.`);}
export function createServer(){return http.createServer(async(req,res)=>{try{if(req.method==='OPTIONS')return send(res,204,null);const url=new URL(req.url||'/','http://localhost');if(req.method==='GET'&&url.pathname==='/health')return send(res,200,{status:'ok',app:META.app});if(req.method==='GET'&&url.pathname==='/openapi.json')return send(res,200,OPENAPI);for(const r of META.routes){if(r.method!==req.method)continue;const params=matchRoute(r.path,url.pathname);if(params)return await handle(r,params,req,res);}return error(res,404,'route_not_found',`No route matches ${req.method} ${url.pathname}.`);}catch(e){return error(res,e.status||500,e.code||'internal_error',e.message||'Internal error.');}});}
export function start(port=Number.parseInt(process.env.PORT||'3000',10)){const s=createServer();s.listen(port,()=>{console.log(`${META.app} listening on http://localhost:${port}`);console.log(`OpenAPI: http://localhost:${port}/openapi.json`);});return s;}
if(import.meta.url===pathToFileURL(process.argv[1]).href){start();}
'''


def generated_server(spec: ApiSpec, routes: list[dict[str, Any]]) -> str:
    meta = {"app": spec.app_name, "database": spec.database.to_dict() if spec.database else {"kind": "memory", "value": "memory"}, "records": {k: v.to_dict() for k, v in spec.records.items()}, "routes": routes}
    return SERVER_TEMPLATE.replace("@@META@@", json.dumps(meta, indent=2, sort_keys=True)).replace("@@OPENAPI@@", json.dumps(build_openapi(spec), indent=2, sort_keys=True))


def sample_value(field: ApiField) -> Any:
    if field.type_name in {"Text", "Email"}:
        if field.name == "title":
            return "Build Hayulo"
        if field.name == "email" or field.type_name == "Email":
            return "ada@example.com"
        return f"sample {field.name}"
    if field.type_name == "Bool":
        return False
    if field.type_name in {"Int", "Float"} or field.type_name.startswith("Id<"):
        return 1
    if field.type_name == "Time":
        return "2026-01-01T00:00:00.000Z"
    return f"sample {field.name}"


def generated_smoke_test(spec: ApiSpec, routes: list[dict[str, Any]]) -> str:
    list_route = next((r for r in routes if r["action"] == "list"), None)
    get_route = next((r for r in routes if r["action"] == "get"), None)
    create_route = next((r for r in routes if r["action"] == "create"), None)
    done_route = next((r for r in routes if r["action"] == "mark_done"), None)
    delete_route = next((r for r in routes if r["action"] == "delete"), None)
    body: dict[str, Any] = {}
    if create_route and create_route.get("body", {}).get("type") in spec.records:
        for field in spec.records[create_route["body"]["type"]].fields:
            body[field.name] = sample_value(field)
    tests = [
        "const health=await request('/health');",
        "assert.equal(health.status,200);",
        f"assert.equal(health.body.app,{json.dumps(spec.app_name)});",
        "const openapi=await request('/openapi.json');",
        "assert.equal(openapi.status,200);",
        "assert.equal(openapi.body.openapi,'3.1.0');",
    ]
    if list_route:
        tests += [
            f"assert.ok(openapi.body.paths[{json.dumps(list_route['path'])}]);",
            f"const listBefore=await request({json.dumps(list_route['path'])});",
            "assert.equal(listBefore.status,200);",
            "assert.ok(Array.isArray(listBefore.body));",
        ]
    if create_route:
        tests += [
            f"const invalidCreate=await request({json.dumps(create_route['path'])},{{method:'POST',body:{{}}}});",
            "assert.equal(invalidCreate.status,400);",
            f"const created=await request({json.dumps(create_route['path'])},{{method:'POST',body:{json.dumps(body)}}});",
            "assert.equal(created.status,201);",
            "const createdId=created.body.id;",
        ]
        for key, value in body.items():
            tests.append(f"assert.equal(created.body[{json.dumps(key)}],{json.dumps(value)});")
    else:
        tests += ["const createdId=1;"]
    if get_route:
        get_path = get_route['path'].replace('{id}', '${createdId}')
        tests += [f"const fetched=await request(`{get_path}`);", "assert.equal(fetched.status,200);", "assert.equal(fetched.body.id,createdId);"]
    if list_route and create_route:
        tests += [f"const listAfter=await request({json.dumps(list_route['path'])});", "assert.equal(listAfter.status,200);", "assert.ok(listAfter.body.some(item=>item.id===createdId));"]
    if done_route:
        done_path = done_route['path'].replace('{id}', '${createdId}')
        tests += [f"const markedDone=await request(`{done_path}`,{{method:'PATCH'}});", "assert.equal(markedDone.status,200);", "assert.equal(markedDone.body.done,true);"]
    if delete_route:
        delete_path = delete_route['path'].replace('{id}', '${createdId}')
        tests += [f"const deleted=await request(`{delete_path}`,{{method:'DELETE'}});", "assert.equal(deleted.status,204);"]
    if get_route and delete_route:
        get_path = get_route['path'].replace('{id}', '${createdId}')
        tests += [f"const fetchedAfterDelete=await request(`{get_path}`);", "assert.equal(fetchedAfterDelete.status,404);"]
    joined = "\n  ".join(tests)
    return """// Generated by Hayulo. Basic integration smoke test.
import assert from 'node:assert/strict';
import http from 'node:http';
import { createServer } from './server.mjs';
const server=createServer();
await new Promise(resolve=>server.listen(0,resolve));
const port=server.address().port;
function request(path,options={}){return new Promise((resolve,reject)=>{const payload=options.body===undefined?null:JSON.stringify(options.body);const req=http.request({hostname:'127.0.0.1',port,path,method:options.method||'GET',headers:{'content-type':'application/json',...(payload?{'content-length':Buffer.byteLength(payload)}:{})}},res=>{let text='';res.on('data',chunk=>text+=chunk);res.on('end',()=>resolve({status:res.statusCode,body:text?JSON.parse(text):null}));});req.on('error',reject);if(payload)req.write(payload);req.end();});}
try{
  @@TESTS@@
  console.log('Hayulo generated API smoke test passed.');
}finally{
  await new Promise(resolve=>server.close(resolve));
}
""".replace("@@TESTS@@", joined)

def generated_readme(spec: ApiSpec) -> str:
    routes = "\n".join(f"- `{r.method} {r.path}` -> `{r.response_type}`" for r in spec.routes)
    return f"""# {spec.app_name} generated API

This directory was generated by Hayulo from a `.hayulo` API source file.

## Run

```bash
npm start
```

## Test

```bash
npm test
```

## Endpoints

- `GET /health`
- `GET /openapi.json`
{routes}

This first MVP generator uses Node.js built-ins and a local JSON file store so the REST API can run without external dependencies. Future Hayulo versions can target TypeScript, Hono/Fastify, real SQLite migrations, auth adapters, and deployment targets.
"""
