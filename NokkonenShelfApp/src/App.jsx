import { useEffect, useState } from 'react'
import { useDispatch, useSelector } from 'react-redux'
import ShelfArea from './components/ShelfArea'
import BookDetails from './components/BookDetails'
import AddBookModal from './components/AddBookModal'
import ConfirmModal from './components/ConfirmModal'
import LandingPage from './components/LandingPage'
import { THEMES } from './themes'
import { addBookcase, deleteBookcase, setBookcaseTheme, fetchUserBookcases } from './store/booksSlice'
import { fetchCurrentUser, logout } from './store/authSlice'
import './App.css'

export default function App() {
  const dispatch = useDispatch()
  const { bookcases, activeIndex, selectedId, status: bookStatus } = useSelector(s => s.books)
  const { user, status: authStatus } = useSelector(s => s.auth)

  useEffect(() => {
    dispatch(fetchCurrentUser())
  }, [dispatch])

  useEffect(() => {
    if (user) dispatch(fetchUserBookcases())
  }, [user, dispatch])

  const [showAddModal, setShowAddModal] = useState(false)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const [isEditing, setIsEditing] = useState(false)

  function handleDeleteConfirmed() {
    dispatch(deleteBookcase())
    setShowDeleteConfirm(false)
    setIsEditing(false)
  }

  if (authStatus === 'idle' || authStatus === 'loading') {
    return <div className="app-loading" />
  }

  if (!user) {
    return <LandingPage />
  }

  if (bookStatus !== 'ready' || !bookcases.length) {
    return <div className="app-loading" />
  }

  const activeBookcase = bookcases[activeIndex]
  const themeId = activeBookcase.themeId
  const theme = THEMES.find(t => t.id === themeId)
  const totalBooks = activeBookcase.shelves.reduce((sum, s) => sum + s.books.length, 0)

  return (
    <div className="app">
      <header className="app-header">
        <h1 className="app-header__title">NokkonenShelf</h1>
        <div className="header-actions">
          <span className="bookcase-counter">{activeIndex + 1} / {bookcases.length}</span>
          <button
            className={`btn ${isEditing ? 'btn--primary' : 'btn--ghost-light'}`}
            onClick={() => setIsEditing(e => !e)}
          >
            {isEditing ? 'Done' : 'Edit'}
          </button>
          <button className="btn btn--primary" onClick={() => setShowAddModal(true)}>
            + Add book
          </button>
          <div className="header-user">
            {user.avatar_url && (
              <img className="header-user__avatar" src={user.avatar_url} alt="" />
            )}
            <span className="header-user__name">{user.username}</span>
            <button className="btn btn--ghost-light" onClick={() => dispatch(logout())}>
              Log out
            </button>
          </div>
        </div>
      </header>

      {isEditing && (
        <div className="edit-toolbar">
          <div className="edit-toolbar__swatches">
            <span className="edit-toolbar__label">Style</span>
            {THEMES.map(t => (
              <button
                key={t.id}
                className={`theme-swatch ${themeId === t.id ? 'theme-swatch--active' : ''}`}
                style={{ backgroundColor: t.swatch }}
                title={t.label}
                onClick={() => dispatch(setBookcaseTheme({ bookcaseIndex: activeIndex, themeId: t.id }))}
                aria-label={t.label}
              />
            ))}
          </div>
          <div className="edit-toolbar__actions">
            <button className="btn btn--ghost-light" onClick={() => dispatch(addBookcase())}>
              + Bookcase
            </button>
            <button
              className="btn btn--danger-outline"
              onClick={() => setShowDeleteConfirm(true)}
              disabled={bookcases.length <= 1}
              title={bookcases.length <= 1 ? 'Cannot delete the only bookcase' : 'Delete this bookcase'}
            >
              Delete bookcase
            </button>
          </div>
        </div>
      )}

      <main className="app-main">
        <ShelfArea theme={theme} isEditing={isEditing} />
      </main>

      <BookDetails key={selectedId ?? 'none'} />

      {showAddModal && (
        <AddBookModal onClose={() => setShowAddModal(false)} />
      )}

      {showDeleteConfirm && (
        <ConfirmModal
          title={`Delete "${activeBookcase.title}"?`}
          message={
            totalBooks > 0
              ? `This bookcase contains ${totalBooks} book${totalBooks !== 1 ? 's' : ''}. Deleting it will permanently remove all of them.`
              : 'This bookcase is empty. It will be permanently removed.'
          }
          confirmLabel="Delete bookcase"
          onConfirm={handleDeleteConfirmed}
          onCancel={() => setShowDeleteConfirm(false)}
        />
      )}
    </div>
  )
}
