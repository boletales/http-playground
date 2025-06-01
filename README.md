## API

### GET `/`

#### レスポンス

操作パネルの html を返します。

### GET `/getcount`

#### レスポンス

```json
{
  "count": 0
}
```

`count`: 現在のカウント値です。

### POST `/addcount`

#### リクエストボディ

```json
{
  "count": 1
}
```

- `count`: 追加するカウント値（正負）です。

#### レスポンス

```json
{
  "count": 1
}
```

- `count`: 変更後ののカウント値です。`

### POST `/resetcount`

#### リクエストボディ

空

#### レスポンス

```json
{
  "count": 0
}
```

- `count`: リセット後のカウント値です。
