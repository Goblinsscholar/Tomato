import { useState, useEffect, useCallback } from 'react';
import { useTagStore } from '@/stores/tagStore';

interface TagSelectorProps {
  selectedTag: string | null;
  onTagChange: (tag: string | null) => void;
}

export function TagSelector({ selectedTag, onTagChange }: TagSelectorProps) {
  const [newTagName, setNewTagName] = useState('');
  const [showInput, setShowInput] = useState(false);
  const { tags, fetch: fetchTags, createTag, deleteTag } = useTagStore();

  useEffect(() => {
    if (tags.length === 0) {
      fetchTags();
    }
  }, [fetchTags, tags.length]);

  // Auto-select first tag when tags load and nothing selected
  useEffect(() => {
    if (!selectedTag && tags.length > 0) {
      onTagChange(tags[0].name);
    }
  }, [tags, selectedTag, onTagChange]);

  const handleAddTag = useCallback(async () => {
    const trimmed = newTagName.trim().slice(0, 30);
    if (!trimmed) return;
    await createTag(trimmed);
    onTagChange(trimmed);
    setNewTagName('');
    setShowInput(false);
  }, [newTagName, createTag, onTagChange]);

  const handleDelete = useCallback(async (e: React.MouseEvent, name: string) => {
    e.stopPropagation();
    await deleteTag(name);
    if (selectedTag === name) {
      // Select another tag or null
      const remaining = tags.filter((t) => t.name !== name);
      onTagChange(remaining.length > 0 ? remaining[0].name : null);
    }
  }, [deleteTag, selectedTag, tags, onTagChange]);

  const { getColor } = useTagStore();

  // Empty state: no tags yet
  if (tags.length === 0 && !showInput) {
    return (
      <div className="flex flex-col items-center gap-2">
        <p className="text-xs text-muted-foreground">还没有标签，添加第一个标签开始专注</p>
        <button
          type="button"
          onClick={() => setShowInput(true)}
          className="rounded-full px-3 py-1 text-[11px] font-medium bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          + 添加标签
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      {/* Tag buttons */}
      {tags.map((tag) => (
        <button
          key={tag.name}
          type="button"
          onClick={() => onTagChange(tag.name)}
          className={`inline-flex items-center gap-0.5 rounded-full px-2.5 py-1 text-[11px] font-medium transition-colors ${
            selectedTag === tag.name
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-muted-foreground hover:bg-muted/80'
          }`}
        >
          <span
            className="inline-block h-2 w-2 rounded-full"
            style={{ backgroundColor: getColor(tag.name) }}
          />
          {tag.name}
          <span
            onClick={(e) => handleDelete(e, tag.name)}
            className="ml-0.5 hover:bg-foreground/20 rounded-full p-0.5 leading-none"
            title={`删除 "${tag.name}"`}
          >
            ×
          </span>
        </button>
      ))}

      {/* Add tag button or input */}
      {!showInput ? (
        <button
          type="button"
          onClick={() => setShowInput(true)}
          className="rounded-full px-2.5 py-1 text-[11px] font-medium text-muted-foreground hover:bg-muted/80 transition-colors"
        >
          + 添加标签
        </button>
      ) : (
        <div className="flex items-center gap-1">
          <input
            type="text"
            value={newTagName}
            onChange={(e) => setNewTagName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleAddTag();
              if (e.key === 'Escape') {
                setShowInput(false);
                setNewTagName('');
              }
            }}
            placeholder="标签名称"
            maxLength={30}
            className="h-8 w-24 rounded-md border border-input bg-background px-2 text-[11px]"
            autoFocus
          />
          <button
            type="button"
            onClick={handleAddTag}
            disabled={!newTagName.trim()}
            className="rounded-full px-2 py-1 text-[11px] font-medium bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
          >
            确定
          </button>
        </div>
      )}
    </div>
  );
}
