import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { useDispatch } from 'react-redux'
import { selectBook } from '../store/booksSlice'

export default function Book({ book, isSelected }) {
  const dispatch = useDispatch()
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: book.id,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
    zIndex: isDragging ? 999 : 'auto',
  }

  function handleClick(e) {
    if (isDragging) return
    e.stopPropagation()
    dispatch(selectBook(book.id))
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`book ${isSelected ? 'book--selected' : ''}`}
      onClick={handleClick}
      {...attributes}
      {...listeners}
    >
      <div
        className="book__spine"
        style={{ backgroundColor: book.color, height: book.height }}
      >
        <span className="book__title">{book.title}</span>
        <span className="book__author">{book.author}</span>
      </div>
      <div className="book__tooltip">
        <span className="book__tooltip-title">{book.title}</span>
        {book.author && <span className="book__tooltip-author">by {book.author}</span>}
      </div>
    </div>
  )
}
