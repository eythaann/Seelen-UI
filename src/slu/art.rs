use slu_ipc::commands::{ArtCli, ArtVariant};

pub fn process(cmd: ArtCli) {
    match cmd.variant {
        ArtVariant::SeelenLogo => println!("{SEELEN_LOGO_ASCII}"),
        ArtVariant::SeelenLogoSmall => println!("{SEELEN_LOGO_ASCII_SMALL}"),
    }
}

static SEELEN_LOGO_ASCII: &str = r#"
                   .      .&     _,x&"``
                    & .   &'  ;.&&'
              &.  . &.&     .0&&&;&""`
         .    '&  &.&&&  .&&&&&'
       .&         ;&&& &&&&&'
      &&          &&&&&&&&     &&&
     0&    .     &&&&&&&&""
    &&   .0     &&&&&&&
   0&& .&'     &&&&&&
  :&&&&&    . &&&&&
  0&&&&    & &&&&&
  &&&&'   &&&&&&&               .&&&x&
  &&&&   :&&&&&0.&'        , .&&&&&&&&&&;.
  &&&&.  &&&&&&&&        .&&&&&&&&&&'               .
  0&&&&  &&&&&&&       ,&&&&&&&&&&&&                &
  :&&&&; &&&&&0       ,;&&&&&&&&&&&             ;  .0
   0&&&&&&&&&&0     ,;&&&&&&&&&&&&&             &  &;
    0&&&&&&&&&&0   :',;".&&&&&&".&             && &0
     0&&&&&&&&&0  ',;',&&&&&" ,&'             &&&&0
      0&&&&&&&&&0 ,x&&&&" .&&&              &&&&0
        0&&&&&& .&&&&"'''"&&"&&            &&&&&0
         0&& .&&;``       `&: :&         &&&&&&0
            &"' &&&&&&&&   &"& &"&   &&&&&&&&0
              0&&&&&&&&&&&&&&&&&&&&&&&&&0
                 0&&&&&&&&&&&&&&&&&&&0
                      0&&&&&&&&&0         Seelen Corp
"#;

static SEELEN_LOGO_ASCII_SMALL: &str = r#"
           xx x                    
           xxxxxx xxxxxxxX         
    xx  xx xxxxxxxxxxxxx           
   xx    xxxxxxxxxx                
  xx  x   xxxxxxx  xxx             
 xxxxx  xxxxxxx                    
xxxx  xxxxxxX                      
xxx  xxxxxx          xxxx          
xxx  xxxxxx       xxxxxxxxxxx     x
xxxxxxxxxx     xxxxxxxxxx         x
xxxxxxxxxx    xxxxxxxxx        x xx
 xxxxxxxxx  xxxxxxxxxxx       xxxxx
 xxxxxxxxx xxxxxxxxxxx       xxxxx 
  xxxxxxxx xxxxxxxxxx       xxxxx  
   xxxxxxxxxxxxxxxxx      xxxxxx   
    Xxxxxxxxxxx  xxxxxxxxxxxxx     
        xxxxxxxxxxxxxxxxxxxx       
          xxxxxxxxxxxxxxx          
"#;
