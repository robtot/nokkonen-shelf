import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'

export const fetchCurrentUser = createAsyncThunk(
  'auth/fetchCurrentUser',
  async (_, { rejectWithValue }) => {
    const res = await fetch('/auth/me')
    if (!res.ok) return rejectWithValue(null)
    return res.json()
  }
)

export const logout = createAsyncThunk('auth/logout', async () => {
  await fetch('/auth/logout', { method: 'POST' })
})

const authSlice = createSlice({
  name: 'auth',
  initialState: {
    user: null,     // { username, email, avatar_url } | null
    status: 'idle', // 'idle' | 'loading' | 'success'
  },
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchCurrentUser.pending, (state) => {
        state.status = 'loading'
      })
      .addCase(fetchCurrentUser.fulfilled, (state, action) => {
        state.status = 'success'
        state.user = action.payload
      })
      .addCase(fetchCurrentUser.rejected, (state) => {
        state.status = 'success'
        state.user = null
      })
      .addCase(logout.fulfilled, (state) => {
        state.user = null
      })
  },
})

export default authSlice.reducer