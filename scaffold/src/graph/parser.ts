const parser = (jsonStr: string): SparqlQueryRes => {
    try {
        const json = JSON.parse(jsonStr)
        const { results: { bindings = [] } = {} } = json || {}
        let data = []
        if (Array.isArray(bindings)) {
            data = bindings.map(item => {
                const newItem: any = {}
                Object.keys(item).forEach(key => {
                    newItem[key] = item[key]?.value || ''
                })
                return newItem
            })
        }
        return {
            originalData: json,
            data,
        } as SparqlQueryRes
    } catch (e) {
        throw e
    }
}

export default parser