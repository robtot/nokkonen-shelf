import { useDispatch, useSelector } from 'react-redux'
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import {
  SortableContext,
  horizontalListSortingStrategy,
  arrayMove,
} from '@dnd-kit/sortable'
import { reorderBooks, deselectBook } from '../store/booksSlice'
import Book from './Book'

export default function Bookshelf() {
  const dispatch = useDispatch()
  const { items: books, selectedId } = useSelector(s => s.books)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  )

  function handleDragEnd(event) {
    const { active, over } = event
    if (over && active.id !== over.id) {
      const oldIndex = books.findIndex(b => b.id === active.id)
      const newIndex = books.findIndex(b => b.id === over.id)
      dispatch(reorderBooks(arrayMove(books, oldIndex, newIndex)))
    }
  }

  function handleShelfClick() {
    dispatch(deselectBook())
  }

  return (
    <div className="shelf-wrapper" onClick={handleShelfClick}>
      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
        <SortableContext items={books.map(b => b.id)} strategy={horizontalListSortingStrategy}>
          <div className="shelf">
            <div className="shelf__books">
              {books.map(book => (
                <Book key={book.id} book={book} isSelected={book.id === selectedId} />
              ))}
            </div>
            <div className="shelf__plank" />
          </div>
        </SortableContext>
      </DndContext>
    </div>
  )
}
