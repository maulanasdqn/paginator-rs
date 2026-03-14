---
title: Query Parameters
description: HTTP query parameter format for web framework integrations
---

All web framework integrations (Axum, Rocket, Actix-web) support the same query parameter format.

## Basic Pagination

```
GET /api/users?page=2&per_page=20
```

| Parameter | Type | Default | Range |
|-----------|------|---------|-------|
| `page` | `u32` | `1` | >= 1 |
| `per_page` | `u32` | `20` | 1-100 |

## Sorting

```
GET /api/users?sort_by=name&sort_direction=asc
GET /api/users?sort_by=created_at&sort_direction=desc
```

| Parameter | Values |
|-----------|--------|
| `sort_by` | Any field name |
| `sort_direction` | `asc` or `desc` |

## Filtering

Filters use the format `field:operator:value`:

```
GET /api/users?filter=status:eq:active&filter=age:gt:18
```

Multiple `filter` parameters are combined with AND logic.

### Filter Format Examples

| Filter | Operator | Description |
|--------|----------|-------------|
| `status:eq:active` | Equal | `status = 'active'` |
| `age:ne:0` | Not equal | `age != 0` |
| `age:gt:18` | Greater than | `age > 18` |
| `age:lt:65` | Less than | `age < 65` |
| `age:gte:18` | Greater or equal | `age >= 18` |
| `age:lte:65` | Less or equal | `age <= 65` |
| `name:like:%john%` | LIKE | `name LIKE '%john%'` |
| `name:ilike:%john%` | ILIKE | `name ILIKE '%john%'` |
| `role:in:admin,mod` | IN | `role IN ('admin', 'mod')` |
| `role:not_in:guest` | NOT IN | `role NOT IN ('guest')` |
| `age:between:18,65` | BETWEEN | `age BETWEEN 18 AND 65` |
| `deleted_at:is_null` | IS NULL | `deleted_at IS NULL` |
| `email:is_not_null` | IS NOT NULL | `email IS NOT NULL` |
| `bio:contains:rust` | Contains | `bio LIKE '%rust%'` |

## Search

```
GET /api/users?search=john&search_fields=name,email,bio
```

| Parameter | Description |
|-----------|-------------|
| `search` | Search query text |
| `search_fields` | Comma-separated list of fields to search |

## Combined Example

```
GET /api/users?page=1&per_page=10&filter=status:eq:active&filter=age:gt:18&search=developer&search_fields=title,bio&sort_by=created_at&sort_direction=desc
```
