import { useDroppable } from '@dnd-kit/core'
import { SortableContext, horizontalListSortingStrategy } from '@dnd-kit/sortable'
import { useDispatch } from 'react-redux'
import { deselectBook, removeShelf } from '../store/booksSlice'
import Book from './Book'

export default function Shelf({ shelfId, shelfNumber, books, selectedId, isEditing, bookcaseIndex }) {
  const dispatch = useDispatch()
  const { setNodeRef } = useDroppable({ id: `shelf-${shelfId}` })

  const isEmpty = books.length === 0

  function handleClick() {
    dispatch(deselectBook())
  }

  function handleRemove() {
    dispatch(removeShelf({ bookcaseIndex, shelfId }))
  }

  return (
    <div className="shelf-wrapper" onClick={handleClick}>
      <div className="shelf" ref={setNodeRef}>
        <SortableContext items={books.map(b => b.id)} strategy={horizontalListSortingStrategy}>
          <div className="shelf__books">
            {isEditing && (
              <div className="shelf__edit-controls">
                <span className="shelf__label">Shelf {shelfNumber}</span>
                <button
                  className="shelf__remove-btn"
                  onClick={e => { e.stopPropagation(); handleRemove() }}
                  disabled={!isEmpty}
                  title={isEmpty ? 'Remove shelf' : 'Remove all books first'}
                  aria-label="Remove shelf"
                >
                  ✕
                </button>
              </div>
            )}
            {books.map(book => (
              <Book key={book.id} book={book} isSelected={book.id === selectedId} />
            ))}
          </div>
        </SortableContext>
        <div className="shelf__plank" />
      </div>
    </div>
  )
}
