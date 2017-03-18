extern crate iron;
extern crate router;
#[macro_use]
extern crate hyper;
extern crate params;
extern crate rustc_serialize;
extern crate time;
#[macro_use]
extern crate mysql;
use std::io::prelude::*;
use std::fs::File;
use iron::prelude::*;
use router::Router;
use hyper::header::{ContentType};
use iron::status;
use rustc_serialize::json;
use mysql::Pool;
use std::str;
use std::env;
use std::collections::HashMap;
use hyper::header::HeaderFormat;
use std::fmt;
static NAME:  &'static str = "Rusty Distrochooser";
static VERSION:  &'static str = "3.0.0";
header! { (Server, "Server") => [String] }
static mut LANG: i32 = 1;
fn main() {
    println!("Starting {} {}...",NAME, VERSION);
    let mut router = Router::new();
    router.get("/", index, "index"); 
    router.get("/distributions/:lang/", distributions,"distros"); 
    router.get("/distribution/:id/:lang/", distribution,"distro"); 
    router.get("/questions/:lang/", questions,"questions"); 
    router.get("/i18n/:lang/", i18n,"i18n"); 
    router.get("/newvisitor/", newvisitor,"newvisitor"); 
    router.get("/get/:lang/", get,"get"); 
    router.post("/newresult/",newresult,"newresult");
    Iron::new(router).http("localhost:3000").unwrap();
}
/**
* Helpers
*/
fn connect_database() -> Pool{
    if let Some(arg1) = env::args().nth(1) {
        let mut f = File::open(arg1).unwrap(); 
        let mut data = String::new();
        f.read_to_string(&mut data);
        let pool = Pool::new(data.as_str()).unwrap();
        return pool;
    }else{
        return Pool::new("").unwrap();
    }
}
fn middleware(request: &mut Request){
    let target: String = format!("{:?}",request.url.path()[0]).replace("\"","");
    let client = request.remote_addr.ip(); //TODO: Censor IP
    language(request);
}
fn language(request: &mut Request){
    let ref lang:&str = request.extensions.get::<Router>().unwrap().find("lang").unwrap_or("/");
    unsafe{
         match lang.as_ref() {
            "de" => LANG = 1,
            _ => LANG = 2,
        }
    }
    /*
    let get =  request.get_ref::<Params>();
    for item in get {
        println!("{:?}",item); //get
    }
    */
}

fn get_id(request: &mut Request) -> i32{
    return request.extensions.get::<Router>().unwrap().find("id").unwrap().parse::<i32>().unwrap();
}

fn get_distros(pool: Pool) -> Vec<Distro>{
    unsafe {
        let query: String = format!("Select d.Id as id ,d.Name as name,d.Homepage as homepage,d.Image as image, (
        Select dd.Description as description from phisco_ldc3.dictDistribution dd where  dd.DistributionId = d.Id and dd.LanguageId = {} limit 1
        ) as description,d.ImageSource as imagesource,d.TextSource as textsource,d.ColorCode as colorcode,d.Characteristica  as characteristica from  phisco_ldc3.Distribution d",LANG); 
        let mut distros: Vec<Distro> = Vec::new();
        let mut conn = pool.get_conn().unwrap();
        let result = conn.prep_exec(query,()).unwrap();
        for row in result {
            let mut r = row.unwrap();
            let mut d = Distro{
                    id:  r.take("id").unwrap(),
                    name:r.take("name").unwrap(),
                    description: r.take("description").unwrap(),
                    homepage: r.take("homepage").unwrap(),
                    image: r.take("image").unwrap(),
                    imagesource: r.take("imagesource").unwrap(),
                    textsource: r.take("textsource").unwrap(),
                    colorcode: r.take("colorcode").unwrap(),
                    tags:  Vec::new()
            };
            d.tags = d.get_tags(r.take("characteristica").unwrap());
            distros.push(d);
        }
        return distros;
    }
}

fn get_questions(pool: Pool) -> Vec<Question>{
    unsafe {
        let query: String = format!("Select q.Id as id,q.OrderIndex, dq.Text as text,q.Single as single, dq.Help as help,q.* from phisco_ldc3.Question q INNER JOIN phisco_ldc3.dictQuestion dq
			ON LanguageId = {} and QuestionId= q.Id order by q.OrderIndex",LANG); 
        let mut questions: Vec<Question> = Vec::new();
        let mut conn = pool.get_conn().unwrap();
        let result = conn.prep_exec(query,()).unwrap();
        let mut i:i32 = 1;
        for row in result {
           let mut r = row.unwrap();
           let mut id: i32 = r.take("id").unwrap();
           let mut q = Question{             
                buttontext: String::new(),
                help: r.take("help").unwrap(),
                id: id, 
                answers: get_answers(id),
                important: false,
                number: i,
                singleanswer: r.take("single").unwrap(),
                text: r.take("text").unwrap()
           };
           questions.push(q);
           i +=1;
        }
        return questions;
    }
}

fn get_answers(id: i32) -> Vec<Answer>{
    unsafe {
        let mut pool: Pool = connect_database();
        let query: String = format!("Select a.Id as id,(
							Select da.Text from phisco_ldc3.dictAnswer da where da.AnswerId = a.Id and da.LanguageId = {}
						)as text,a.Tags,a.NoTags,a.IsText as istext from phisco_ldc3.Answer a where a.QuestionId = {}",LANG,id); 
        let mut answers: Vec<Answer> = Vec::new();
        let mut conn = pool.get_conn().unwrap();
        let result = conn.prep_exec(query,()).unwrap();
        for row in result {
           let mut r = row.unwrap();
           let tags: String = r.take("Tags").unwrap();
           let notags: String= r.take("NoTags").unwrap();
           let mut a = Answer{              
                id: r.take("id").unwrap(),
                text: r.take("text").unwrap(),
                notags: json::decode(&notags).unwrap(),
                tags: json::decode(&tags).unwrap(),
                image: String::new(),
                istext: r.take("istext").unwrap(),
                selected: false
           };
           answers.push(a);
        }
        return answers;
    }
}

fn get_i18n(p: Pool) -> HashMap<String,i18nValue>{
    unsafe {
        let mut pool: Pool = connect_database();
        let query: String = format!("Select Text,Val,Val as Name from phisco_ldc3.dictSystem where LanguageId =  {}",LANG); 
        let mut values = HashMap::new();
        let mut conn = pool.get_conn().unwrap();
        let result = conn.prep_exec(query,()).unwrap();
        for row in result {
           let mut r = row.unwrap();
           let text: String = r.take("Text").unwrap();
           let val: String = r.take("Val").unwrap();
           let name: String = r.take("Name").unwrap();
           values.insert(name,i18nValue::new(text,val));
        }
        return values;
    }
}

fn get_distro(pool: Pool, id: i32) -> APIResult{
    let distros: Vec<Distro> = get_distros(pool);
    for distro in distros{
        if distro.id == id{
            return Ok(distro)
        }
    }
    return Err(APIError::DistroNotFound)
}
fn get_response(body: String) -> Response{
    let mut resp = Response::with((status::Ok, body.to_owned()));
    resp.headers.set(ContentType::json());
    resp.headers.set(Server(format!("{} {}",NAME,VERSION).to_owned()));
    //TODO: CACHING
    //let mut sha = Sha256::new();
    //sha.input_str(body.as_str());
    //println!("{}", sha.result_str());
    return resp;
}

fn new_visitor(p: Pool,request: &mut Request) -> i32{
    let mut useragent: String = String::new();
    let mut referer: String = String::new();
    match  request.headers.get::<iron::headers::UserAgent>() {
        Some(x) => useragent = format!("{}",x),
        None    => useragent = String::new(),
    }
    match  request.headers.get::<iron::headers::Referer>() {
        Some(x) => referer = format!("{}",x),
        None    => referer = String::new(),
    }
    let tm = time::now();
    let time = format!("{}",tm.strftime("%Y-%m-%d %H:%M:%S").unwrap());
    //TODO: DNT
    let query: String = String::from("Insert into phisco_ldc3.Visitor (Date, Referrer, Useragent, DNT) VALUES (:time,:ref,:ua,:dnt)");
    p.prep_exec(query,(time,referer ,useragent,true)).unwrap();

    //return visitor id
    let max_id: String = String::from("Select max(Id) as id from phisco_ldc3.Visitor");
    let mut id: i32 = 0;
    let mut conn = p.get_conn().unwrap();
    let result = conn.prep_exec(max_id,()).unwrap();
    for row in result {
        let mut r = row.unwrap();
        id = r.take("id").unwrap();
    }
    return id;
}
/**
* Routes
*/
fn index(_request: &mut Request) -> IronResult<Response> {    
    middleware(_request);
    Ok(get_response(String::from("I'm an rusty API.")))
}
fn get(_request: &mut Request) -> IronResult<Response>{
    middleware(_request);
    let mut result: get = get{
        questions: get_questions(connect_database()),
        distros: get_distros(connect_database()),
        i18n: get_i18n(connect_database()),
        visitor: new_visitor(connect_database(),_request)
    };
    Ok(get_response(String::from(json::encode(&result).unwrap())))
}
fn newvisitor(_request: &mut Request) -> IronResult<Response> {    
    middleware(_request);
    let id: i32 = new_visitor(connect_database(),_request);
    let body: String = format!("{}",id);
    Ok(get_response(body))
}
fn newresult(_request: &mut Request) -> IronResult<Response> {    
    middleware(_request);
   
    /**
    * Parse distributions
    */

   // 
    /**
    * Parse tags
    */
   // let tags = rustc_serialize::json::Json::from_str(&tags_json).unwrap();
   // let tagsObj = tags.as_object().unwrap();
    /*for (key, value) in obj.iter() {
        println!("{}: {}", key, value);
    }
    */
    /**
    * Parse answers
    */
   
   // let answers: Vec<String> = json::decode(&answers_json).unwrap();
    /**
    * Parse important flag
    */
   
    //let important: Vec<String> = json::decode(&important_json).unwrap();

    let mut useragent: String = String::new();
    match  _request.headers.get::<iron::headers::UserAgent>() {
        Some(x) => useragent = format!("{}",x),
        None    => useragent = String::new(),
    }

    let params = _request.get_ref::<params::Params>().unwrap();
    let mut distro_json: String = format!("{:?}",params["distros"]);
    distro_json = String::from(distro_json.trim_matches('"').replace("\\",""));

    let mut tags_json: String = format!("{:?}",params["tags"]);
    tags_json = String::from(tags_json.trim_matches('"').replace("\\",""));

    let mut answers_json: String = format!("{:?}",params["answers"]);
    answers_json = String::from(answers_json.trim_matches('"').replace("\\",""));
    
    let mut important_json: String = format!("{:?}",params["important"]); 
    important_json = String::from(important_json.trim_matches('"').replace("\\",""));

    let tm = time::now();
    let time = format!("{}",tm.strftime("%Y-%m-%d %H:%M:%S").unwrap());
    let mut p: Pool = connect_database();
    let query: String = String::from("Insert into phisco_ldc3.Result (Date,UserAgent,Tags, Answers,Important) Values(CURRENT_TIMESTAMP,:ua,:tags,:answers,:important)");
    p.prep_exec(query,(useragent,tags_json,answers_json,important_json)).unwrap();

    //return result id
    let max_id: String = String::from("Select max(Id) as id from phisco_ldc3.Result");
    let mut id: i32 = 0;
    let mut conn = p.get_conn().unwrap();
    let result = conn.prep_exec(max_id,()).unwrap();
    for row in result {
        let mut r = row.unwrap();
        id = r.take("id").unwrap();
    }

    let distros: Vec<Distro> = json::decode(&distro_json).unwrap();

    for distro in distros{
        let addResult: String = String::from("Insert into phisco_ldc3.ResultDistro (DistroId,ResultId) Values(:distro,:result)");
        p.prep_exec(addResult,(distro.id,id)).unwrap();
    }

    Ok(get_response(format!("{:?}",id)))
}

fn distributions(_request: &mut Request) -> IronResult<Response> {
    middleware(_request);
    let distros: Vec<Distro> = get_distros(connect_database());
    Ok(get_response(String::from(json::encode(&distros).unwrap())))
}
fn distribution(_request: &mut Request) -> IronResult<Response> {
    middleware(_request);
    let id: i32 = get_id(_request);
    let raw = get_distro(connect_database(),id);
    let mut distro: Option<Distro> = None;
    match raw{
        Ok(n) => distro = Some(n),
        Err(_) => distro = None
    };
    let resp;
    if distro.is_none(){
        resp = Response::with((status::NotFound,"Not found"));
    }else{
        resp = get_response(String::from(json::encode(&distro).unwrap()));
    }
    return Ok(resp);
}
fn questions(_request: &mut Request) -> IronResult<Response>{
    middleware(_request);
    let questions: Vec<Question> = get_questions(connect_database());
    Ok(get_response(String::from(json::encode(&questions).unwrap())))
}
fn i18n(_request: &mut Request) -> IronResult<Response>{
    middleware(_request);
    let translation: HashMap<String,i18nValue> = get_i18n(connect_database());
    Ok(get_response(String::from(json::encode(&translation).unwrap())))
}
/**
* Structs
*/
#[derive(RustcDecodable, RustcEncodable)]
pub struct Distro {
    id: i32,
    name: String,
    description: String,
    homepage: String,
    image: String,
    imagesource: String,
    textsource: String,
    colorcode: String,
    tags: Vec<String>
}

impl Distro{
    fn get_tags(&self,s: String) -> Vec<String> {
        let v: Vec<String> = json::decode(&s.to_owned()).unwrap();
        return v;
    }
}

#[derive(RustcDecodable, RustcEncodable)]
struct Question{
    answers: Vec<Answer>,
    buttontext: String,
    help: String,
    id: i32,
    important: bool,
    number: i32,
    singleanswer: bool,
    text: String
}

#[derive(RustcDecodable, RustcEncodable)]
struct Answer{
    id: i32,
    image: String,
    istext: bool,
    selected: bool,
    tags: Vec<String>,
    notags: Vec<String>,
    text: String
}

#[derive(RustcDecodable, RustcEncodable,Hash, Eq, PartialEq, Debug)]
struct i18nValue{
    text: String,
    val: String
}


#[derive(RustcDecodable, RustcEncodable)]
struct get{
    distros: Vec<Distro>,
    questions: Vec<Question>,
    i18n: HashMap<String,i18nValue>,
    visitor: i32
}

impl i18nValue {
    fn new(t: String, v: String) -> i18nValue {
        i18nValue { text: t.to_string(), val: v.to_string() }
    }
}

/**
* Enums
*/
#[derive(RustcDecodable, RustcEncodable)]
pub enum APIError {
    DistroNotFound
}
/**
* Types
*/
pub type APIResult = Result<Distro, APIError>;