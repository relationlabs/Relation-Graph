import React, { useState, useCallback } from 'react'
import Button from './components/button'
import Loading from './components/loading'
import { sparqlQuery } from './graph'
import './App.css'

function App() {
  const [loading, setLoading] = useState(false)
  const [list, setList] = useState<any>([])
  const queryData = useCallback(async () => {
    setLoading(true)
    const queryRes = await sparqlQuery({
      host: 'http://localhost',
      port: 9933,
      sparql: 'SELECT ?name ?age  WHERE { :P1001 :name ?name; :age ?age .}',
    })
    const { data = [] } = queryRes || {}
    if (Array.isArray(data)) {
      setList(data)
    }
    
    setLoading(false)
  }, [])
  return (
    <div className="graph-app">
      <div className="header">
        Hello Relation Graph
      </div>
      <div className='actions'>
        <Button onClick={queryData}>
          QUERY
        </Button>
      </div>
      <div className='list'>
        {
          loading ? (
            <div className='loading-wrap'>
              <Loading />
            </div>
          ) : (
            <>
              <div className='list-head'>
                <span>Name</span>
                <span>Age</span>
              </div>
              {list.map((item: any, index: number) => {
                const { name, age } = item
                return (
                  <div key={index} className="list-item">
                    <span>{name}</span>
                    <span>{age}</span>
                  </div>
                )
              })}
            </>
          )
        }
      </div>
    </div>
  )
}

export default App
