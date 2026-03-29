import { useState } from 'react'
import { useDispatch, useSelector } from 'react-redux'
import { addBook } from '../store/booksSlice'

function parseOpenLibrarySlug(url) {
  // e.g. https://openlibrary.org/works/OL262758W/The_Hobbit
  // or   https://openlibrary.org/works/OL262758W
  try {
    const u = new URL(url)
    if (!u.hostname.includes('openlibrary.org')) return null
    const parts = u.pathname.split('/').filter(Boolean)
    // parts: ['works', 'OL262758W', 'The_Hobbit'] or ['books', 'OL...']
    return parts.slice(0, 2).join('/') // 'works/OL262758W'
  } catch {
    return null
  }
}

function titleFromUrl(url) {
  try {
    const u = new URL(url)
    const parts = u.pathname.split('/').filter(Boolean)
    // last segment may be slug like "The_Hobbit"
    const last = parts[parts.length - 1]
    if (last && !last.startsWith('OL')) {
      return last.replace(/_/g, ' ')
    }
  } catch {
    // ignore
  }
  return ''
}

export default function AddBookModal({ onClose }) {
  const dispatch = useDispatch()
  const { bookcases, activeIndex } = useSelector(s => s.books)
  const shelves = bookcases[activeIndex].shelves

  const [url, setUrl] = useState('')
  const [title, setTitle] = useState('')
  const [author, setAuthor] = useState('')
  const [shelfId, setShelfId] = useState(shelves[0]?.id ?? '')
  const [error, setError] = useState('')

  function handleUrlBlur() {
    if (url && !title) {
      const derived = titleFromUrl(url)
      if (derived) setTitle(derived)
    }
  }

  function handleSubmit(e) {
    e.preventDefault()
    if (!title.trim()) {
      setError('Title is required.')
      return
    }
    const slug = url ? parseOpenLibrarySlug(url) : null
    if (url && !slug) {
      setError('Please enter a valid Open Library URL.')
      return
    }
    dispatch(
      addBook({
        id: Date.now().toString(),
        title: title.trim(),
        author: author.trim(),
        openLibraryUrl: url.trim() || null,
        description: '',
        year: null,
        shelfId,
      })
    )
    onClose()
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <div className="modal__header">
          <h2>Add a book</h2>
          <button className="modal__close" onClick={onClose} aria-label="Close">✕</button>
        </div>
        <form className="modal__form" onSubmit={handleSubmit}>
          <label className="modal__label">
            Open Library URL <span className="modal__optional">(optional)</span>
            <input
              className="modal__input"
              type="url"
              placeholder="https://openlibrary.org/works/OL..."
              value={url}
              onChange={e => { setUrl(e.target.value); setError('') }}
              onBlur={handleUrlBlur}
            />
          </label>
          <label className="modal__label">
            Title <span className="modal__required">*</span>
            <input
              className="modal__input"
              type="text"
              placeholder="Book title"
              value={title}
              onChange={e => { setTitle(e.target.value); setError('') }}
            />
          </label>
          <label className="modal__label">
            Author
            <input
              className="modal__input"
              type="text"
              placeholder="Author name"
              value={author}
              onChange={e => setAuthor(e.target.value)}
            />
          </label>
          <label className="modal__label">
            Shelf
            <select
              className="modal__input"
              value={shelfId}
              onChange={e => setShelfId(e.target.value)}
            >
              {shelves.map((shelf, i) => (
                <option key={shelf.id} value={shelf.id}>Shelf {i + 1}</option>
              ))}
            </select>
          </label>
          {error && <p className="modal__error">{error}</p>}
          <div className="modal__actions">
            <button type="button" className="btn btn--ghost" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn btn--primary">Add to shelf</button>
          </div>
        </form>
      </div>
    </div>
  )
}
