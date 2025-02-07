package storage

import (
	"context"
	"encoding/json"
	"github.com/redis/go-redis/v9"
	"time"
)

type CacheStorage interface {
	HSet(ctx context.Context, key, field string, v any, expire time.Duration) error
	HGet(ctx context.Context, key, field string, v any) error
	HDelete(ctx context.Context, key string, fields ...string) error
	HDeleteAll(ctx context.Context, key, pattern string) error
	Del(ctx context.Context, keys ...string) error
}

type RedisCacheStorage struct {
	redis *redis.Client
}

func NewRedisCacheStorage(ctx context.Context, addr string) (*RedisCacheStorage, error) {
	rdb := redis.NewClient(&redis.Options{
		Addr:     addr,
		Password: "",
		DB:       0,
	})
	if _, err := rdb.Ping(ctx).Result(); err != nil {
		return nil, err
	}

	return &RedisCacheStorage{
		redis: rdb,
	}, nil
}

func (r *RedisCacheStorage) HSet(ctx context.Context, key, field string, v any, expire time.Duration) error {
	if v == nil {
		return nil
	}

	jsonString, _ := json.Marshal(v)
	if err := r.redis.HSet(ctx, key, field, string(jsonString)).Err(); err != nil {
		return err
	}

	return r.redis.HExpire(ctx, key, expire, field).Err()
}

func (r *RedisCacheStorage) HGet(ctx context.Context, key, field string, v any) error {
	jsonString, err := r.redis.HGet(ctx, key, field).Result()
	if err != nil {
		return err
	}

	err = json.Unmarshal([]byte(jsonString), v)
	return err
}

func (r *RedisCacheStorage) HDelete(ctx context.Context, key string, fields ...string) error {
	return r.redis.HDel(ctx, key, fields...).Err()
}

func (r *RedisCacheStorage) HDeleteAll(ctx context.Context, key, pattern string) error {
	cursor := uint64(0)

	for {
		fields, nextCursor, err := r.redis.HScan(ctx, key, cursor, pattern, 100).Result()
		if err != nil {
			return err
		}

		if len(fields) > 0 {
			if err := r.redis.HDel(ctx, key, fields...).Err(); err != nil {
				return err
			}
		}

		cursor = nextCursor
		if cursor == 0 {
			break
		}
	}
	return nil
}

func (r *RedisCacheStorage) Del(ctx context.Context, keys ...string) error {
	return r.redis.Del(ctx, keys...).Err()
}
