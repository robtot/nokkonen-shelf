import { useState, useRef, useEffect } from 'react'
import { useDispatch, useSelector } from 'react-redux'
import {
  DndContext,
  DragOverlay,
  closestCenter,
  pointerWithin,
  rectIntersection,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { arrayMove } from '@dnd-kit/sortable'
import { setShelves, addShelf, goNext, goPrev, setBookcaseTitle, saveShelfOrder } from '../store/booksSlice'
import Shelf from './Shelf'

// Prefer pointer-within hits; fall back to rect intersection, then closest center.
// This ensures empty shelves (large droppable, no sortable children) are detected.
function collisionDetection(args) {
  const pointerHits = pointerWithin(args)
  if (pointerHits.length > 0) return pointerHits
  const rectHits = rectIntersection(args)
  if (rectHits.length > 0) return rectHits
  return closestCenter(args)
}

// id is either `shelf-{shelfId}` (from useDroppable) or a book id
function findContainer(id, shelves) {
  const strId = String(id)
  if (strId.startsWith('shelf-')) return strId.slice(6)
  for (const shelf of shelves) {
    if (shelf.books.find(b => b.id === strId)) return shelf.id
  }
  return null
}

export default function ShelfArea({ theme, isEditing }) {
  const dispatch = useDispatch()
  const { bookcases, activeIndex, selectedId } = useSelector(s => s.books)
  const activeBookcase = bookcases[activeIndex]
  const shelves = activeBookcase.shelves

  const [dragActiveId, setDragActiveId] = useState(null)
  const [localShelves, setLocalShelves] = useState(null)

  const saveTimerRef = useRef(null)
  const pendingSaveRef = useRef(null)

  function scheduleSave(bookcaseIndex, finalShelves) {
    pendingSaveRef.current = { bookcaseIndex, shelves: finalShelves }
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current)
    saveTimerRef.current = setTimeout(() => {
      saveTimerRef.current = null
      dispatch(saveShelfOrder(pendingSaveRef.current))
      pendingSaveRef.current = null
    }, 600)
  }

  useEffect(() => {
    function handleBeforeUnload(e) {
      if (!pendingSaveRef.current) return
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current)
        saveTimerRef.current = null
      }
      dispatch(saveShelfOrder(pendingSaveRef.current))
      pendingSaveRef.current = null
      e.preventDefault()
    }
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [dispatch])

  const displayShelves = localShelves ?? shelves
  const dragActiveBook = dragActiveId
    ? displayShelves.flatMap(s => s.books).find(b => b.id === dragActiveId)
    : null

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  )

  function handleDragStart({ active }) {
    setDragActiveId(active.id)
    setLocalShelves(structuredClone(shelves))
  }

  function handleDragOver({ active, over }) {
    if (!over || !localShelves) return

    const activeContainer = findContainer(active.id, localShelves)
    const overContainer = findContainer(over.id, localShelves)

    if (!activeContainer || !overContainer || activeContainer === overContainer) return

    setLocalShelves(prev => {
      const activeShelf = prev.find(s => s.id === activeContainer)
      const overShelf = prev.find(s => s.id === overContainer)
      if (!activeShelf) return prev

      const movedBook = activeShelf.books.find(b => b.id === active.id)
      const overIdx = overShelf.books.findIndex(b => b.id === over.id)
      const insertAt = overIdx >= 0 ? overIdx : overShelf.books.length

      return prev.map(s => {
        if (s.id === activeContainer) return { ...s, books: s.books.filter(b => b.id !== active.id) }
        if (s.id === overContainer) return {
          ...s,
          books: [
            ...s.books.slice(0, insertAt),
            movedBook,
            ...s.books.slice(insertAt),
          ],
        }
        return s
      })
    })
  }

  function handleDragEnd({ active, over }) {
    setDragActiveId(null)

    if (!localShelves) return

    let finalShelves = localShelves

    if (over) {
      const activeContainer = findContainer(active.id, localShelves)
      const overContainer = findContainer(over.id, localShelves)

      if (
        activeContainer &&
        overContainer &&
        activeContainer === overContainer &&
        active.id !== over.id
      ) {
        const shelf = localShelves.find(s => s.id === activeContainer)
        const oldIndex = shelf.books.findIndex(b => b.id === active.id)
        const newIndex = shelf.books.findIndex(b => b.id === over.id)
        if (oldIndex !== -1 && newIndex !== -1) {
          finalShelves = localShelves.map(s =>
            s.id === activeContainer
              ? { ...s, books: arrayMove(s.books, oldIndex, newIndex) }
              : s
          )
        }
      }
    }

    dispatch(setShelves({ bookcaseIndex: activeIndex, shelves: finalShelves }))
    scheduleSave(activeIndex, finalShelves)
    setLocalShelves(null)
  }

  function handleDragCancel() {
    setDragActiveId(null)
    setLocalShelves(null)
  }

  const hasPrev = activeIndex > 0
  const hasNext = activeIndex < bookcases.length - 1

  // ── Inline title editing ──
  const [editingTitle, setEditingTitle] = useState(false)
  const [titleDraft, setTitleDraft] = useState('')
  const titleInputRef = useRef(null)

  function startEditTitle() {
    setTitleDraft(activeBookcase.title)
    setEditingTitle(true)
    setTimeout(() => titleInputRef.current?.select(), 0)
  }

  function commitTitle() {
    const trimmed = titleDraft.trim()
    if (trimmed) {
      dispatch(setBookcaseTitle({ bookcaseIndex: activeIndex, title: trimmed }))
    }
    setEditingTitle(false)
  }

  function handleTitleKeyDown(e) {
    if (e.key === 'Enter') commitTitle()
    if (e.key === 'Escape') setEditingTitle(false)
  }

  return (
    <div className="bookcase-wrapper">
      <div className="bookcase-title-row">
        {editingTitle ? (
          <input
            ref={titleInputRef}
            className="bookcase-title-input"
            value={titleDraft}
            onChange={e => setTitleDraft(e.target.value)}
            onBlur={commitTitle}
            onKeyDown={handleTitleKeyDown}
          />
        ) : (
          <h2 className="bookcase-title" onClick={startEditTitle} title="Click to rename">
            {activeBookcase.title}
          </h2>
        )}
      </div>
    <div className="bookcase-navigator">
      <button
        className="nav-arrow"
        onClick={() => dispatch(goPrev())}
        aria-label="Previous bookcase"
        style={{ visibility: hasPrev ? 'visible' : 'hidden' }}
      >
        ‹
      </button>

      <DndContext
        sensors={sensors}
        collisionDetection={collisionDetection}
        onDragStart={handleDragStart}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
        onDragCancel={handleDragCancel}
      >
        <div className={`bookcase ${isEditing ? 'bookcase--editing' : ''}`} style={theme?.vars}>
          <div className="shelf-area">
            {displayShelves.map((shelf, index) => (
              <Shelf
                key={shelf.id}
                shelfId={shelf.id}
                shelfNumber={index + 1}
                books={shelf.books}
                selectedId={selectedId}
                isEditing={isEditing}
                bookcaseIndex={activeIndex}
              />
            ))}
          </div>
          {isEditing && (
            <button className="add-shelf-btn" onClick={() => dispatch(addShelf())}>
              + Add shelf
            </button>
          )}
        </div>
        <DragOverlay>
          {dragActiveBook && (
            <div className="book book--dragging">
              <div
                className="book__spine"
                style={{ backgroundColor: dragActiveBook.color, height: dragActiveBook.height }}
              >
                <span className="book__title">{dragActiveBook.title}</span>
                <span className="book__author">{dragActiveBook.author}</span>
              </div>
            </div>
          )}
        </DragOverlay>
      </DndContext>

      <button
        className="nav-arrow"
        onClick={() => dispatch(goNext())}
        aria-label="Next bookcase"
        style={{ visibility: hasNext ? 'visible' : 'hidden' }}
      >
        ›
      </button>
    </div>
    </div>
  )
}
