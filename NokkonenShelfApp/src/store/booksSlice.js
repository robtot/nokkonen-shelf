import { createAsyncThunk, createSlice } from '@reduxjs/toolkit'

// ── Visual helpers ─────────────────────────────────────────────────────────────
const BOOK_HEIGHTS = [160, 175, 185, 170, 190, 165, 180, 172, 188, 168]

function bookHeight(id) {
  const sum = [...id].reduce((s, c) => s + c.charCodeAt(0), 0)
  return BOOK_HEIGHTS[sum % BOOK_HEIGHTS.length]
}

// ── Theme persistence (localStorage) ──────────────────────────────────────────
function loadTheme(bookcaseId) {
  return localStorage.getItem(`theme_${bookcaseId}`) ?? 'walnut'
}

function saveTheme(bookcaseId, themeId) {
  localStorage.setItem(`theme_${bookcaseId}`, themeId)
}

// ── API → store mapping ────────────────────────────────────────────────────────
function mapBook(b) {
  return {
    id: b.id,
    title: b.title,
    author: b.author,
    color: b.color,
    height: bookHeight(b.id),
    openLibraryUrl: b.open_library_url ?? null,
  }
}

function mapBookcase(bc) {
  return {
    id: bc.id,
    title: bc.name,
    themeId: loadTheme(bc.id),
    shelves: bc.shelves.map(s => ({
      id: s.id,
      books: s.books.map(mapBook),
    })),
  }
}

// ── Async thunks ───────────────────────────────────────────────────────────────
export const fetchUserBookcases = createAsyncThunk(
  'books/fetchUserBookcases',
  async () => {
    const listRes = await fetch('/api/bookcases')
    if (!listRes.ok) throw new Error('Failed to fetch bookcases')
    const list = await listRes.json()

    const details = await Promise.all(
      list.map(async bc => {
        const res = await fetch(`/api/bookcases/${bc.id}`)
        if (!res.ok) throw new Error(`Failed to fetch bookcase ${bc.id}`)
        return res.json()
      })
    )

    return details.map(mapBookcase)
  }
)

export const addBookcase = createAsyncThunk(
  'books/addBookcase',
  async (_, { getState }) => {
    const count = getState().books.bookcases.length
    const res = await fetch('/api/bookcases', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: `Bookcase ${count + 1}` }),
    })
    if (!res.ok) throw new Error('Failed to create bookcase')
    const bc = await res.json()

    await Promise.all(
      [1, 2, 3].map(() =>
        fetch(`/api/bookcases/${bc.id}/shelves`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name: '' }),
        })
      )
    )

    const detailRes = await fetch(`/api/bookcases/${bc.id}`)
    if (!detailRes.ok) throw new Error('Failed to fetch new bookcase detail')
    return mapBookcase(await detailRes.json())
  }
)

export const deleteBookcase = createAsyncThunk(
  'books/deleteBookcase',
  async (_, { getState }) => {
    const { bookcases, activeIndex } = getState().books
    if (bookcases.length <= 1) return null
    const id = bookcases[activeIndex].id
    const res = await fetch(`/api/bookcases/${id}`, { method: 'DELETE' })
    if (!res.ok) throw new Error('Failed to delete bookcase')
    return id
  }
)

export const addBook = createAsyncThunk(
  'books/addBook',
  async ({ shelfId, title, author, openLibraryUrl }, { getState }) => {
    const { bookcases, activeIndex } = getState().books
    const targetShelfId = shelfId ?? bookcases[activeIndex].shelves[0]?.id
    if (!targetShelfId) throw new Error('No shelf available')

    const res = await fetch(`/api/shelves/${targetShelfId}/books`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title,
        author: author || '',
        open_library_url: openLibraryUrl || null,
      }),
    })
    if (!res.ok) throw new Error('Failed to add book')
    return { shelfId: targetShelfId, book: mapBook(await res.json()) }
  }
)

export const removeBook = createAsyncThunk(
  'books/removeBook',
  async (bookId) => {
    const res = await fetch(`/api/books/${bookId}`, { method: 'DELETE' })
    if (!res.ok) throw new Error('Failed to delete book')
    return bookId
  }
)

export const addShelf = createAsyncThunk(
  'books/addShelf',
  async (_, { getState }) => {
    const { bookcases, activeIndex } = getState().books
    const bookcaseId = bookcases[activeIndex].id
    const res = await fetch(`/api/bookcases/${bookcaseId}/shelves`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: '' }),
    })
    if (!res.ok) throw new Error('Failed to add shelf')
    const shelf = await res.json()
    return { bookcaseIndex: activeIndex, shelf: { id: shelf.id, books: [] } }
  }
)

export const moveBookToBookcase = createAsyncThunk(
  'books/moveBookToBookcase',
  async ({ bookId, targetBookcaseIndex, targetShelfId }) => {
    const res = await fetch(`/api/books/${bookId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ shelf_id: targetShelfId }),
    })
    if (!res.ok) throw new Error('Failed to move book')
    return { bookId, targetBookcaseIndex, targetShelfId }
  }
)

export const saveShelfOrder = createAsyncThunk(
  'books/saveShelfOrder',
  async ({ bookcaseIndex, shelves }, { getState }) => {
    const bookcaseId = getState().books.bookcases[bookcaseIndex].id
    const res = await fetch(`/api/bookcases/${bookcaseId}/books/reorder`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        shelves: shelves.map(s => ({ id: s.id, book_ids: s.books.map(b => b.id) })),
      }),
    })
    if (!res.ok) throw new Error('Failed to save book order')
  }
)

export const removeShelf = createAsyncThunk(
  'books/removeShelf',
  async ({ bookcaseIndex, shelfId }) => {
    const res = await fetch(`/api/shelves/${shelfId}`, { method: 'DELETE' })
    if (!res.ok) throw new Error('Failed to delete shelf')
    return { bookcaseIndex, shelfId }
  }
)

export const setBookcaseTitle = createAsyncThunk(
  'books/setBookcaseTitle',
  async ({ bookcaseIndex, title }, { getState }) => {
    const id = getState().books.bookcases[bookcaseIndex].id
    const res = await fetch(`/api/bookcases/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: title }),
    })
    if (!res.ok) throw new Error('Failed to rename bookcase')
    return { bookcaseIndex, title }
  }
)

// ── Slice ──────────────────────────────────────────────────────────────────────
const booksSlice = createSlice({
  name: 'books',
  initialState: {
    bookcases: [],
    activeIndex: 0,
    selectedId: null,
    status: 'idle', // 'idle' | 'loading' | 'ready'
  },
  reducers: {
    setShelves(state, action) {
      const { bookcaseIndex, shelves } = action.payload
      state.bookcases[bookcaseIndex].shelves = shelves
    },
    setBookcaseTheme(state, action) {
      const { bookcaseIndex, themeId } = action.payload
      state.bookcases[bookcaseIndex].themeId = themeId
      saveTheme(state.bookcases[bookcaseIndex].id, themeId)
    },
    goNext(state) {
      if (state.activeIndex < state.bookcases.length - 1) {
        state.activeIndex++
        state.selectedId = null
      }
    },
    goPrev(state) {
      if (state.activeIndex > 0) {
        state.activeIndex--
        state.selectedId = null
      }
    },
    selectBook(state, action) {
      state.selectedId = action.payload
    },
    deselectBook(state) {
      state.selectedId = null
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchUserBookcases.pending, (state) => {
        state.status = 'loading'
      })
      .addCase(fetchUserBookcases.fulfilled, (state, action) => {
        state.bookcases = action.payload
        state.activeIndex = 0
        state.status = 'ready'
      })
      .addCase(fetchUserBookcases.rejected, (state) => {
        state.status = 'ready'
      })

      .addCase(addBookcase.fulfilled, (state, action) => {
        state.bookcases.push(action.payload)
        state.activeIndex = state.bookcases.length - 1
        state.selectedId = null
      })

      .addCase(deleteBookcase.fulfilled, (state, action) => {
        if (!action.payload) return
        const idx = state.bookcases.findIndex(bc => bc.id === action.payload)
        if (idx !== -1) {
          state.bookcases.splice(idx, 1)
          state.activeIndex = Math.min(state.activeIndex, state.bookcases.length - 1)
          state.selectedId = null
        }
      })

      .addCase(addBook.fulfilled, (state, action) => {
        const { shelfId, book } = action.payload
        for (const bc of state.bookcases) {
          const shelf = bc.shelves.find(s => s.id === shelfId)
          if (shelf) { shelf.books.push(book); break }
        }
      })

      .addCase(removeBook.fulfilled, (state, action) => {
        const bookId = action.payload
        for (const bc of state.bookcases) {
          for (const shelf of bc.shelves) {
            const idx = shelf.books.findIndex(b => b.id === bookId)
            if (idx !== -1) { shelf.books.splice(idx, 1); break }
          }
        }
        if (state.selectedId === bookId) state.selectedId = null
      })

      .addCase(addShelf.fulfilled, (state, action) => {
        const { bookcaseIndex, shelf } = action.payload
        state.bookcases[bookcaseIndex].shelves.push(shelf)
      })

      .addCase(removeShelf.fulfilled, (state, action) => {
        const { bookcaseIndex, shelfId } = action.payload
        const shelves = state.bookcases[bookcaseIndex].shelves
        const idx = shelves.findIndex(s => s.id === shelfId)
        if (idx !== -1) shelves.splice(idx, 1)
      })

      .addCase(setBookcaseTitle.fulfilled, (state, action) => {
        const { bookcaseIndex, title } = action.payload
        state.bookcases[bookcaseIndex].title = title
      })

      .addCase(moveBookToBookcase.fulfilled, (state, action) => {
        const { bookId, targetBookcaseIndex, targetShelfId } = action.payload
        let book = null
        for (const bc of state.bookcases) {
          for (const shelf of bc.shelves) {
            const idx = shelf.books.findIndex(b => b.id === bookId)
            if (idx !== -1) {
              ;[book] = shelf.books.splice(idx, 1)
              break
            }
          }
          if (book) break
        }
        if (!book) return
        const targetBc = state.bookcases[targetBookcaseIndex]
        const targetShelf = targetBc.shelves.find(s => s.id === targetShelfId) ?? targetBc.shelves[0]
        targetShelf.books.push(book)
        state.selectedId = null
      })
  },
})

export const {
  setShelves, setBookcaseTheme,
  goNext, goPrev, selectBook, deselectBook,
} = booksSlice.actions
export default booksSlice.reducer
