import { useState } from 'react'
import { useDispatch, useSelector } from 'react-redux'
import { deselectBook, removeBook, moveBookToBookcase } from '../store/booksSlice'

export default function BookDetails() {
  const dispatch = useDispatch()
  const { bookcases, selectedId } = useSelector(s => s.books)
  const book = bookcases.flatMap(bc => bc.shelves.flatMap(s => s.books)).find(b => b.id === selectedId)

  // Index of the bookcase that currently holds this book
  const currentBcIndex = bookcases.findIndex(bc =>
    bc.shelves.some(s => s.books.some(b => b.id === selectedId))
  )

  const otherBookcases = bookcases
    .map((bc, i) => ({ ...bc, realIndex: i }))
    .filter((_, i) => i !== currentBcIndex)

  const [targetBcIndex, setTargetBcIndex] = useState(() => otherBookcases[0]?.realIndex ?? 0)
  const [targetShelfId, setTargetShelfId] = useState(
    () => bookcases[otherBookcases[0]?.realIndex]?.shelves[0]?.id ?? ''
  )

  if (!book) return null

  function handleTargetBcChange(e) {
    const newBcIndex = Number(e.target.value)
    setTargetBcIndex(newBcIndex)
    setTargetShelfId(bookcases[newBcIndex]?.shelves[0]?.id ?? '')
  }

  function handleMove() {
    dispatch(moveBookToBookcase({ bookId: book.id, targetBookcaseIndex: targetBcIndex, targetShelfId }))
  }

  function handleClose() {
    dispatch(deselectBook())
  }

  const targetBookcase = bookcases[targetBcIndex]
  const canMove = otherBookcases.length > 0

  return (
    <div className="book-details-overlay" onClick={handleClose}>
      <div className="book-details" onClick={e => e.stopPropagation()}>
        <div className="book-details__header" style={{ backgroundColor: book.color }}>
          <h2 className="book-details__title">{book.title}</h2>
          <button className="book-details__close" onClick={handleClose} aria-label="Close">✕</button>
        </div>
        <div className="book-details__body">
          <p className="book-details__author">by {book.author}</p>
          {book.year && <p className="book-details__year">{book.year}</p>}
          {book.description && <p className="book-details__description">{book.description}</p>}
          {book.openLibraryUrl && (
            <a
              className="book-details__link"
              href={book.openLibraryUrl}
              target="_blank"
              rel="noreferrer"
            >
              View on Open Library →
            </a>
          )}

          {canMove && (
            <div className="book-details__move">
              <span className="book-details__move-label">Send to</span>
              <div className="book-details__move-selects">
                <select
                  className="modal__input book-details__move-select"
                  value={targetBcIndex}
                  onChange={handleTargetBcChange}
                >
                  {otherBookcases.map(bc => (
                    <option key={bc.id} value={bc.realIndex}>{bc.title}</option>
                  ))}
                </select>
                <select
                  className="modal__input book-details__move-select"
                  value={targetShelfId}
                  onChange={e => setTargetShelfId(e.target.value)}
                >
                  {targetBookcase?.shelves.map((s, i) => (
                    <option key={s.id} value={s.id}>Shelf {i + 1}</option>
                  ))}
                </select>
              </div>
              <button className="btn btn--primary book-details__move-btn" onClick={handleMove}>
                Move
              </button>
            </div>
          )}

          <div className="book-details__footer">
            <button className="book-details__remove" onClick={() => dispatch(removeBook(book.id))}>
              Remove from shelf
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
