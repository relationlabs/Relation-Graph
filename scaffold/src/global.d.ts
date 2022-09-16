type SparqlQueryParams = {
    host: string;
    sparql: string;
    port?: number|string;
}

type SparqlQueryRes = {
    originalData: any;
    data: any[];
}