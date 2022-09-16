import React from 'react'
import loadingSvg from '../../loading.svg'
import './index.css'

const Loading = () => {
  return (
    <div className='rel-loading'>
      <img src={loadingSvg} alt='loading...' />
    </div>
  )
}

export default Loading