import parser from './parser'

const sparqlQuery = ({
    host,
    sparql,
    port = 80,
}: SparqlQueryParams): Promise<SparqlQueryRes|void>|undefined => {
    console.assert(host && sparql, 'require host and sparql')
    if (host && sparql) {
        const body = {
            id: 1,
            jsonrpc: '2.0',
            method: 'sparql_query',
            params: [sparql],
        }
        return fetch(`${host}:${port}`, {
            method: 'POST',
            mode: 'cors',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(body),
        }).then(response => response.json()).then(json => {
            const { result } = json || {}
            const parsedResult: any = parser(result)
            return parsedResult
        }).catch(e => console.error(e))
    }
}

export default sparqlQuery